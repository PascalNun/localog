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
use std::time::Duration;

/// One downloadable whisper.cpp ggml model. Checksums and sizes were verified
/// locally against the published files before being recorded here.
struct ModelSpec {
    id: &'static str,
    file_name: &'static str,
    url: &'static str,
    sha256: &'static str,
    byte_count: u64,
    /// What the model is for. Both kinds live in one table because they are the
    /// same problem — a large file fetched once, checksummed, and kept on the
    /// device — and giving generation its own downloader would mean two of
    /// everything: staging, resuming, the disk check, the progress reporting.
    kind: ModelKind,
    /// Which licence this is offered under, as an identifier the interface has
    /// words for. Not a sentence: the licence a model carries is shown before it
    /// is downloaded and listed afterwards, and both of those are the interface's
    /// to word.
    licence: &'static str,
    /// Where the licence itself can be read. Shown rather than summarised —
    /// paraphrasing somebody's licence terms is not this application's place.
    licence_url: &'static str,
}

/// What a downloaded model does.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ModelKind {
    /// Turns audio into a transcript.
    Transcription,
    /// Separates the voices in a recording.
    Speakers,
    /// Writes the protocol.
    Generation,
}

/// Quality presets, most to least common. Exact model is hidden behind Advanced.
pub(crate) const PRESETS: &[(&str, &str)] = &[
    ("fast", "tiny"),
    ("balanced", "base"),
    ("accurate", "medium"),
];

pub(crate) const DEFAULT_PRESET: &str = "balanced";

/// Models the diariser needs. Both are small next to a transcription model, and
/// both are required together: segmentation finds where voices change, embedding
/// describes each voice so they can be grouped.
pub(crate) const DIARISATION_MODELS: &[&str] = &["speaker-segmentation", "speaker-embedding"];

const MODELS: &[ModelSpec] = &[
    ModelSpec {
        id: "speaker-segmentation",
        file_name: "speaker-segmentation.onnx",
        url: "https://huggingface.co/csukuangfj/sherpa-onnx-pyannote-segmentation-3-0/resolve/main/model.onnx",
        sha256: "220ad67ca923bef2fa91f2390c786097bf305bceb5e261d4af67b38e938e1079",
        byte_count: 5_992_913,
        kind: ModelKind::Speakers,
        licence: "mit",
        licence_url: "https://github.com/k2-fsa/sherpa-onnx/blob/master/LICENSE",
    },
    ModelSpec {
        id: "speaker-embedding",
        file_name: "speaker-embedding.onnx",
        url: "https://huggingface.co/csukuangfj/speaker-embedding-models/resolve/main/3dspeaker_speech_eres2net_base_sv_zh-cn_3dspeaker_16k.onnx",
        sha256: "1a331345f04805badbb495c775a6ddffcdd1a732567d5ec8b3d5749e3c7a5e4b",
        byte_count: 39_593_761,
        kind: ModelKind::Speakers,
        licence: "mit",
        licence_url: "https://github.com/k2-fsa/sherpa-onnx/blob/master/LICENSE",
    },
    ModelSpec {
        id: "tiny",
        file_name: "ggml-tiny.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin",
        sha256: "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21",
        byte_count: 77_691_713,
        kind: ModelKind::Transcription,
        licence: "mit",
        licence_url: "https://github.com/ggml-org/whisper.cpp/blob/master/LICENSE",
    },
    ModelSpec {
        id: "base",
        file_name: "ggml-base.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin",
        sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
        byte_count: 147_951_465,
        kind: ModelKind::Transcription,
        licence: "mit",
        licence_url: "https://github.com/ggml-org/whisper.cpp/blob/master/LICENSE",
    },
    ModelSpec {
        id: "medium",
        file_name: "ggml-medium.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin",
        sha256: "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208",
        byte_count: 1_533_763_059,
        kind: ModelKind::Transcription,
        licence: "mit",
        licence_url: "https://github.com/ggml-org/whisper.cpp/blob/master/LICENSE",
    },
    // The model that writes the protocol, and the one recommended first: it kept
    // 27 to 31 of a reference meeting's 35 stated figures across three runs, where
    // the next best kept as few as 6.
    //
    // Google's own quantisation-aware build rather than a community quantisation.
    // QAT is trained to be quantised, so it holds up at four bits far better than
    // the same weights squeezed afterwards — and holding up means keeping numbers,
    // which is the measure this model was chosen on.
    //
    // The size and the checksum are what the repository reports, read from it
    // rather than estimated. Gemma is offered under Google's own terms rather than
    // Apache, whatever a repository tag says, so the licence named here is the one
    // somebody would actually be agreeing to.
    ModelSpec {
        id: "gemma4-12b",
        file_name: "gemma-4-12b-it-qat-q4_0.gguf",
        url: "https://huggingface.co/google/gemma-4-12B-it-qat-q4_0-gguf/resolve/main/gemma-4-12b-it-qat-q4_0.gguf",
        sha256: "93567e57a8fe10b23569b9d9ec38cd005deedf71e29477c421a4b83f418a538b",
        byte_count: 6_975_879_296,
        kind: ModelKind::Generation,
        licence: "gemma",
        licence_url: "https://ai.google.dev/gemma/terms",
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
            Self::UnknownModel => write!(formatter, "modelUnknown"),
            Self::Cancelled => write!(formatter, "downloadCancelled"),
            Self::NotEnoughSpace { needed, available } => write!(
                formatter,
                "notEnoughSpace:needs {}, {} free",
                human_bytes(*needed),
                human_bytes(*available)
            ),
            Self::Network(message) => {
                write!(formatter, "modelNotDownloaded:{message}")
            }
            Self::VerifyFailed => write!(formatter, "downloadCorrupt"),
            Self::Io(message) => write!(formatter, "modelNotSaved:{message}"),
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

/// What a model is, for the screen that has to say so before it is downloaded.
///
/// Serialised for the interface rather than read there: the URL, the checksum and
/// the licence are facts about the file, and the interface's job is to show them,
/// not to hold a second copy that can disagree.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModelTerms {
    pub model_id: String,
    pub kind: ModelKind,
    pub byte_count: u64,
    /// An identifier the interface has words for — "mit", "gemma" — never a
    /// sentence. The words are the interface's.
    pub licence: String,
    /// Where the terms themselves can be read.
    pub licence_url: String,
    pub installed: bool,
}

/// Everything known about one model's terms, or nothing if it is not catalogued.
pub(crate) fn terms(root: &Path, model_id: &str) -> Option<ModelTerms> {
    let model = spec(model_id)?;
    Some(ModelTerms {
        model_id: model.id.to_string(),
        kind: model.kind,
        byte_count: model.byte_count,
        licence: model.licence.to_string(),
        licence_url: model.licence_url.to_string(),
        installed: installed_model_path(root, model_id).is_some(),
    })
}

/// The terms of everything catalogued, for the page that lists them.
pub(crate) fn every_model_terms(root: &Path) -> Vec<ModelTerms> {
    MODELS
        .iter()
        .filter_map(|model| terms(root, model.id))
        .collect()
}

/// The models that write protocols, most recommended first.
///
/// Order is the catalogue's, and the catalogue's order is what was measured. The
/// first entry is what a new installation is offered without being asked to
/// choose — the same way transcription offers Balanced.
pub(crate) fn generation_models(root: &Path) -> Vec<ModelTerms> {
    MODELS
        .iter()
        .filter(|model| model.kind == ModelKind::Generation)
        .filter_map(|model| terms(root, model.id))
        .collect()
}

/// The generation model to use: the one asked for if it is here, otherwise the
/// first catalogued one that is.
///
/// Returns nothing when none is installed, which is a normal first run rather
/// than a fault — the start screen says so before anybody reaches generation.
pub(crate) fn generation_model_path(root: &Path, preferred: Option<&str>) -> Option<PathBuf> {
    if let Some(wanted) = preferred
        && let Some(path) = installed_model_path(root, wanted)
        && spec(wanted).is_some_and(|model| model.kind == ModelKind::Generation)
    {
        return Some(path);
    }
    MODELS
        .iter()
        .filter(|model| model.kind == ModelKind::Generation)
        .find_map(|model| installed_model_path(root, model.id))
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

/// Where the diariser's models are, once both have been downloaded. Returns
/// `None` unless every one of them is present, because the diariser needs them
/// together and a partial set would fail at the point of use.
pub(crate) fn diarisation_model_paths(root: &Path) -> Option<(PathBuf, PathBuf)> {
    let segmentation = installed_model_path(root, "speaker-segmentation")?;
    let embedding = installed_model_path(root, "speaker-embedding")?;
    Some((segmentation, embedding))
}

/// Total download size of whatever the diariser is still missing, so the user can
/// be told the cost before agreeing to it.
pub(crate) fn diarisation_download_bytes(root: &Path) -> u64 {
    DIARISATION_MODELS
        .iter()
        .filter(|id| installed_model_path(root, id).is_none())
        .filter_map(|id| spec(id))
        .map(|model| model.byte_count)
        .sum()
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

/// Remove an installed model to reclaim space, including any staged remnant.
/// Missing is a no-op success.
pub(crate) fn remove_model(root: &Path, model_id: &str) -> Result<(), ModelError> {
    let model = spec(model_id).ok_or(ModelError::UnknownModel)?;
    let directory = models_dir(root);
    let _ = fs::remove_file(directory.join(format!("{}.part", model.file_name)));
    match fs::remove_file(directory.join(model.file_name)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ModelError::Io(error.to_string())),
    }
}

/// Drop staged downloads abandoned by a crash or force quit. A `.part` file is never
/// resumable, so keeping it would only consume space invisibly.
pub(crate) fn discard_staged_downloads(root: &Path) {
    let directory = models_dir(root);
    let Ok(entries) = fs::read_dir(&directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|value| value == "part") {
            let _ = fs::remove_file(path);
        }
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
        // No global deadline: a 1.5 GB model legitimately takes minutes. Per-operation
        // deadlines still ensure a stalled transfer fails instead of hanging forever.
        .timeout_global(None)
        .timeout_connect(Some(Duration::from_secs(30)))
        .timeout_recv_response(Some(Duration::from_secs(60)))
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
        // One decimal, so a near-miss never reads as "needs 1 GB, 1 GB free".
        format!("{value:.1} {}", UNITS[unit])
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
    fn diarisation_needs_every_model_before_it_can_run() {
        let temporary = tempdir().unwrap();
        let root = temporary.path();
        fs::create_dir_all(models_dir(root)).unwrap();
        assert!(diarisation_model_paths(root).is_none());
        assert_eq!(diarisation_download_bytes(root), 5_992_913 + 39_593_761);

        // One of two present is still not usable, and the remaining cost reflects it.
        let seg = models_dir(root).join("speaker-segmentation.onnx");
        fs::File::create(&seg).unwrap().set_len(5_992_913).unwrap();
        assert!(diarisation_model_paths(root).is_none());
        assert_eq!(diarisation_download_bytes(root), 39_593_761);

        let emb = models_dir(root).join("speaker-embedding.onnx");
        fs::File::create(&emb).unwrap().set_len(39_593_761).unwrap();
        assert_eq!(diarisation_model_paths(root), Some((seg, emb)));
        assert_eq!(diarisation_download_bytes(root), 0);
    }

    #[test]
    fn diarisation_models_are_not_offered_as_transcription_qualities() {
        let temporary = tempdir().unwrap();
        let capability = capability(temporary.path(), "balanced");
        for preset in &capability.presets {
            assert!(
                !DIARISATION_MODELS.contains(&preset.model_id.as_str()),
                "{} is not a transcription quality",
                preset.model_id
            );
        }
    }

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
    fn staged_downloads_are_discarded_and_removal_takes_the_remnant() {
        let temporary = tempdir().unwrap();
        let root = temporary.path();
        let directory = models_dir(root);
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("ggml-base.bin.part"), b"abandoned").unwrap();
        fs::write(directory.join("ggml-tiny.bin.part"), b"abandoned").unwrap();
        discard_staged_downloads(root);
        assert!(!directory.join("ggml-base.bin.part").exists());
        assert!(!directory.join("ggml-tiny.bin.part").exists());

        // remove_model also clears a remnant left beside a real file.
        fs::write(directory.join("ggml-base.bin.part"), b"abandoned").unwrap();
        remove_model(root, "base").unwrap();
        assert!(!directory.join("ggml-base.bin.part").exists());
    }

    #[test]
    fn space_message_distinguishes_near_miss_sizes() {
        let error = ModelError::NotEnoughSpace {
            needed: 1_533_763_059,
            available: 1_400_000_000,
        };
        let message = error.to_string();
        // Whole-unit rounding used to render this as "needs 1 GB, 1 GB free".
        assert!(message.contains("1.4 GB"), "{message}");
        assert!(message.contains("1.3 GB"), "{message}");
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
    /// Ignored by default: performs real HTTPS requests. Run with
    /// `cargo test --lib -- --ignored downloads_the_speaker_models`
    #[test]
    #[ignore]
    fn downloads_the_speaker_models() {
        let temporary = tempdir().unwrap();
        let root = temporary.path();
        for model_id in DIARISATION_MODELS {
            download_model(root, model_id, &AtomicBool::new(false), |_| {})
                .unwrap_or_else(|error| panic!("{model_id} must download: {error}"));
        }
        let (segmentation, embedding) =
            diarisation_model_paths(root).expect("both models must be installed");
        assert_eq!(fs::metadata(&segmentation).unwrap().len(), 5_992_913);
        assert_eq!(fs::metadata(&embedding).unwrap().len(), 39_593_761);
        assert_eq!(diarisation_download_bytes(root), 0);
    }

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

#[cfg(test)]
mod the_catalogue_terms {
    use super::*;

    /// Every model states a licence and where to read it.
    ///
    /// A downloaded file whose terms nobody can name is exactly what this
    /// application should not do to somebody who chose it for being local.
    #[test]
    fn every_model_names_its_licence_and_where_to_read_it() {
        for model in MODELS {
            assert!(!model.licence.is_empty(), "{} has no licence", model.id);
            assert!(
                model.licence_url.starts_with("https://"),
                "{} has no licence to read: {:?}",
                model.id,
                model.licence_url
            );
        }
    }

    /// The checksum is the whole reason a download can be trusted, so it must be
    /// a real one rather than a placeholder somebody meant to fill in.
    #[test]
    fn every_checksum_is_a_sha256() {
        for model in MODELS {
            assert_eq!(model.sha256.len(), 64, "{} has no sha256", model.id);
            assert!(
                model.sha256.chars().all(|glyph| glyph.is_ascii_hexdigit()),
                "{} has a checksum that is not hexadecimal",
                model.id
            );
            assert!(model.byte_count > 0, "{} has no size", model.id);
        }
    }

    /// There is a model that writes protocols, and the first one is the one the
    /// measurements chose. A catalogue that lost it would leave a fresh
    /// installation with nothing to offer.
    #[test]
    fn a_generation_model_is_catalogued_and_recommended_first() {
        let generation: Vec<&ModelSpec> = MODELS
            .iter()
            .filter(|model| model.kind == ModelKind::Generation)
            .collect();
        assert!(!generation.is_empty(), "nothing writes protocols");
        assert_eq!(generation[0].id, "gemma4-12b");
    }

    /// Nothing is installed on a fresh workspace, and asking must answer that
    /// rather than failing.
    #[test]
    fn a_fresh_workspace_has_no_generation_model() {
        let root = std::env::temp_dir().join(format!("localog-terms-{}", std::process::id()));
        let _ = fs::create_dir_all(&root);
        assert!(generation_model_path(&root, None).is_none());
        assert!(generation_model_path(&root, Some("gemma4-12b")).is_none());
        let listed = generation_models(&root);
        assert!(!listed.is_empty(), "the catalogue is still listed");
        assert!(listed.iter().all(|model| !model.installed));
        let _ = fs::remove_dir_all(&root);
    }

    /// A transcription model is never handed to the generation runtime, however
    /// it is asked for. They are different files for different programs.
    #[test]
    fn a_transcription_model_is_not_offered_for_generation() {
        let root = std::env::temp_dir().join(format!("localog-kinds-{}", std::process::id()));
        let _ = fs::create_dir_all(&root);
        assert!(generation_model_path(&root, Some("base")).is_none());
        let _ = fs::remove_dir_all(&root);
    }
}
