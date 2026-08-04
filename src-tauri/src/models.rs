//! Managed transcription models: quality presets plus consent-gated, verified,
//! on-demand download into app-managed storage. The user chooses a quality; the
//! exact model stays an Advanced detail. Every download is checksum-verified and
//! installed atomically, so an interrupted download never appears as a ready model.

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fmt::{Display, Formatter};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// One downloadable whisper.cpp ggml model. Checksums and sizes were verified
/// locally against the published files before being recorded here.
struct ModelSpec {
    id: &'static str,
    file_name: &'static str,
    url: &'static str,
    sha256: &'static str,
    byte_count: u64,
}

/// Quality presets, most to least common. Exact model is hidden behind Advanced.
pub(crate) const PRESETS: &[(&str, &str)] = &[
    ("fast", "tiny"),
    ("balanced", "base"),
    ("accurate", "medium"),
];

pub(crate) const DEFAULT_PRESET: &str = "balanced";

const MODELS: &[ModelSpec] = &[
    ModelSpec {
        id: "tiny",
        file_name: "ggml-tiny.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin",
        sha256: "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21",
        byte_count: 77_691_713,
    },
    ModelSpec {
        id: "base",
        file_name: "ggml-base.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin",
        sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
        byte_count: 147_951_465,
    },
    ModelSpec {
        id: "medium",
        file_name: "ggml-medium.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin",
        sha256: "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208",
        byte_count: 1_533_763_059,
    },
];

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ModelError {
    UnknownModel,
    Cancelled,
    NotEnoughSpace { needed: u64, available: u64 },
    Network(String),
    VerifyFailed,
    Io(String),
}

impl Display for ModelError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownModel => write!(formatter, "That transcription model is not recognised."),
            Self::Cancelled => write!(formatter, "The download was cancelled."),
            Self::NotEnoughSpace { needed, available } => write!(
                formatter,
                "Not enough space for this model (needs {}, {} free).",
                human_bytes(*needed),
                human_bytes(*available)
            ),
            Self::Network(message) => {
                write!(formatter, "The model could not be downloaded: {message}")
            }
            Self::VerifyFailed => write!(
                formatter,
                "The download was incomplete or corrupt and was discarded."
            ),
            Self::Io(message) => write!(formatter, "The model could not be saved: {message}"),
        }
    }
}

impl From<std::io::Error> for ModelError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

/// One preset row for the Transcription settings UI.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PresetStatus {
    pub preset: String,
    pub model_id: String,
    pub byte_count: u64,
    pub installed: bool,
}

/// The whole transcription capability the UI needs: which quality is selected and
/// which presets are ready or need a download.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TranscriptionCapability {
    pub selected_preset: String,
    pub presets: Vec<PresetStatus>,
}

fn spec(model_id: &str) -> Option<&'static ModelSpec> {
    MODELS.iter().find(|model| model.id == model_id)
}

pub(crate) fn preset_model_id(preset: &str) -> Option<&'static str> {
    PRESETS
        .iter()
        .find(|(name, _)| *name == preset)
        .map(|(_, model)| *model)
}

pub(crate) fn is_known_preset(preset: &str) -> bool {
    preset_model_id(preset).is_some()
}

fn models_dir(root: &Path) -> PathBuf {
    root.join("models")
}

/// Path to an installed model, verified only by presence and exact size here.
/// Content is checksum-verified on download and rehashed before inference.
pub(crate) fn installed_model_path(root: &Path, model_id: &str) -> Option<PathBuf> {
    let model = spec(model_id)?;
    let path = models_dir(root).join(model.file_name);
    let matches_size = fs::metadata(&path)
        .map(|meta| meta.len() == model.byte_count)
        .unwrap_or(false);
    matches_size.then_some(path)
}

/// The model file backing a chosen preset, if it is installed.
pub(crate) fn model_path_for_preset(root: &Path, preset: &str) -> Option<PathBuf> {
    installed_model_path(root, preset_model_id(preset)?)
}

pub(crate) fn capability(root: &Path, selected_preset: &str) -> TranscriptionCapability {
    let presets = PRESETS
        .iter()
        .filter_map(|(preset, model_id)| {
            spec(model_id).map(|model| PresetStatus {
                preset: (*preset).to_string(),
                model_id: (*model_id).to_string(),
                byte_count: model.byte_count,
                installed: installed_model_path(root, model_id).is_some(),
            })
        })
        .collect();
    TranscriptionCapability {
        selected_preset: selected_preset.to_string(),
        presets,
    }
}

/// Remove an installed model to reclaim space. Missing is a no-op success.
pub(crate) fn remove_model(root: &Path, model_id: &str) -> Result<(), ModelError> {
    let model = spec(model_id).ok_or(ModelError::UnknownModel)?;
    let path = models_dir(root).join(model.file_name);
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ModelError::Io(error.to_string())),
    }
}

/// Download a model into app-managed storage: stream to a staged file, hash while
/// writing, verify size + digest, then atomically install. Reports 0..=100 percent
/// and observes cancellation between chunks. An interrupted download leaves no
/// visible model — the staged file is removed and the final path is never written
/// until the bytes are verified.
pub(crate) fn download_model(
    root: &Path,
    model_id: &str,
    cancellation: &AtomicBool,
    mut progress: impl FnMut(u8),
) -> Result<(), ModelError> {
    let model = spec(model_id).ok_or(ModelError::UnknownModel)?;
    let directory = models_dir(root);
    fs::create_dir_all(&directory)?;

    if installed_model_path(root, model_id).is_some() {
        progress(100);
        return Ok(());
    }

    if let Some(available) = available_bytes(&directory)
        && available < model.byte_count
    {
        return Err(ModelError::NotEnoughSpace {
            needed: model.byte_count,
            available,
        });
    }

    let staged = directory.join(format!("{}.part", model.file_name));
    let _ = fs::remove_file(&staged);
    let result = stream_to_staged(model, &staged, cancellation, &mut progress);
    if result.is_err() {
        let _ = fs::remove_file(&staged);
        return result;
    }

    // Durably install the verified file, then reveal it at its final path.
    let final_path = directory.join(model.file_name);
    if let Err(error) = File::open(&staged).and_then(|file| file.sync_all()) {
        let _ = fs::remove_file(&staged);
        return Err(ModelError::Io(error.to_string()));
    }
    if let Err(error) = fs::rename(&staged, &final_path) {
        let _ = fs::remove_file(&staged);
        return Err(ModelError::Io(error.to_string()));
    }
    progress(100);
    Ok(())
}

fn stream_to_staged(
    model: &ModelSpec,
    staged: &Path,
    cancellation: &AtomicBool,
    progress: &mut impl FnMut(u8),
) -> Result<(), ModelError> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        // No global deadline: large models legitimately take minutes. Responsiveness
        // comes from polling cancellation between chunks, not from a hard timeout.
        .timeout_global(None)
        .build()
        .into();
    let response = agent
        .get(model.url)
        .call()
        .map_err(|error| ModelError::Network(truncate(&error.to_string(), 200)))?;
    let mut reader = response.into_parts().1.into_reader();

    let mut file = File::create(staged)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1024 * 1024];
    let mut written: u64 = 0;
    let mut last_percent = 0u8;
    progress(0);
    loop {
        if cancellation.load(Ordering::Acquire) {
            return Err(ModelError::Cancelled);
        }
        let read = reader
            .read(&mut buffer)
            .map_err(|error| ModelError::Network(truncate(&error.to_string(), 200)))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        file.write_all(&buffer[..read])?;
        written += read as u64;
        if model.byte_count > 0 {
            // f64 ratio keeps this a plain proportion; byte_count is non-zero here.
            let percent =
                ((written.min(model.byte_count) as f64 / model.byte_count as f64) * 100.0) as u8;
            if percent != last_percent {
                last_percent = percent;
                progress(percent.min(99));
            }
        }
    }
    file.flush()?;

    let digest = format!("{:x}", hasher.finalize());
    if written != model.byte_count || digest != model.sha256 {
        return Err(ModelError::VerifyFailed);
    }
    Ok(())
}

fn truncate(text: &str, max: usize) -> String {
    if text.len() <= max {
        text.to_string()
    } else {
        text.chars().take(max).collect()
    }
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.0} {}", UNITS[unit])
    }
}

#[cfg(unix)]
fn available_bytes(path: &Path) -> Option<u64> {
    use std::os::unix::ffi::OsStrExt;
    let c_path = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) } == 0 {
        Some(stat.f_bavail as u64 * stat.f_frsize as u64)
    } else {
        None
    }
}

#[cfg(not(unix))]
fn available_bytes(_path: &Path) -> Option<u64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn presets_map_to_known_models() {
        assert_eq!(preset_model_id("fast"), Some("tiny"));
        assert_eq!(preset_model_id("balanced"), Some("base"));
        assert_eq!(preset_model_id("accurate"), Some("medium"));
        assert_eq!(preset_model_id("nonsense"), None);
        assert!(is_known_preset(DEFAULT_PRESET));
        // Every preset must reference a real registry entry.
        for (_, model_id) in PRESETS {
            assert!(spec(model_id).is_some(), "missing spec for {model_id}");
        }
    }

    #[test]
    fn installed_detection_requires_exact_size() {
        let temporary = tempdir().unwrap();
        let root = temporary.path();
        fs::create_dir_all(models_dir(root)).unwrap();
        let path = models_dir(root).join("ggml-base.bin");
        // Wrong size must not count as installed.
        fs::write(&path, b"not a real model").unwrap();
        assert!(installed_model_path(root, "base").is_none());
        assert!(model_path_for_preset(root, "balanced").is_none());
    }

    #[test]
    fn capability_reports_presets_and_selection() {
        let temporary = tempdir().unwrap();
        let capability = capability(temporary.path(), "accurate");
        assert_eq!(capability.selected_preset, "accurate");
        assert_eq!(capability.presets.len(), PRESETS.len());
        assert!(capability.presets.iter().all(|preset| !preset.installed));
        let fast = capability
            .presets
            .iter()
            .find(|preset| preset.preset == "fast")
            .unwrap();
        assert_eq!(fast.model_id, "tiny");
        assert_eq!(fast.byte_count, 77_691_713);
    }

    #[test]
    fn removing_a_missing_model_is_a_no_op() {
        let temporary = tempdir().unwrap();
        assert_eq!(remove_model(temporary.path(), "base"), Ok(()));
        assert_eq!(
            remove_model(temporary.path(), "unknown"),
            Err(ModelError::UnknownModel)
        );
    }

    #[test]
    fn download_rejects_an_unknown_model() {
        let temporary = tempdir().unwrap();
        let result = download_model(
            temporary.path(),
            "gigantic",
            &AtomicBool::new(false),
            |_| {},
        );
        assert_eq!(result, Err(ModelError::UnknownModel));
    }

    #[test]
    fn recorded_digests_are_lowercase_hex_of_the_right_length() {
        for model in MODELS {
            assert_eq!(model.sha256.len(), 64, "{} digest length", model.id);
            assert!(
                model
                    .sha256
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
                "{} digest format",
                model.id
            );
        }
    }
}

#[cfg(test)]
mod network_tests {
    use super::*;
    use tempfile::tempdir;

    /// Ignored by default: performs a real HTTPS request. Run explicitly with
    /// `cargo test -- --ignored downloads_tiny` to verify the TLS + verify path.
    #[test]
    #[ignore]
    fn downloads_and_verifies_the_tiny_model() {
        let temporary = tempdir().unwrap();
        let root = temporary.path();
        let mut last = 0u8;
        download_model(root, "tiny", &AtomicBool::new(false), |percent| {
            last = percent;
        })
        .expect("tiny model must download and verify");
        assert_eq!(last, 100);
        let path = installed_model_path(root, "tiny").expect("model must be installed");
        assert_eq!(fs::metadata(&path).unwrap().len(), 77_691_713);
        // No staged remnant may survive a successful install.
        assert!(!root.join("models/ggml-tiny.bin.part").exists());
    }
}
