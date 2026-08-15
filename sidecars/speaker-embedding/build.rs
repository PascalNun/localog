// Link the sherpa-onnx C API, statically where the build offers it.
//
// Static is strongly preferred: a sidecar that links a library which happens to be
// on the build machine runs there and fails on somebody else's, and shipping a
// dylib beside an `externalBin` means another thing to place, sign and notarise.
// A self-contained binary is the same shape as the whisper.cpp sidecar already is.
//
// `SHERPA_ONNX_LIB` is required and points at a directory of built libraries. The
// build refuses without it rather than guessing at a path, because a path that
// happens to be right on one machine is a confusing failure on every other one.

use std::path::Path;

fn main() {
    println!("cargo:rerun-if-env-changed=SHERPA_ONNX_LIB");
    let Ok(directory) = std::env::var("SHERPA_ONNX_LIB") else {
        panic!(
            "Set SHERPA_ONNX_LIB to a directory holding the built sherpa-onnx \
             libraries. scripts/build-embedding-sidecar.sh does this for you."
        );
    };
    let path = Path::new(&directory);
    if !path.is_dir() {
        panic!("SHERPA_ONNX_LIB is not a directory: {directory}");
    }
    println!("cargo:rustc-link-search=native={directory}");

    let statics = archives(path);
    if statics.iter().any(|name| name == "sherpa-onnx-c-api") {
        // Order matters to the linker: the C API calls into the core, the core
        // into the feature and model libraries, and everything into onnxruntime.
        for name in &statics {
            println!("cargo:rustc-link-lib=static={name}");
        }
        // A C++ library needs its runtime, and onnxruntime needs the platform's
        // own frameworks.
        if cfg!(target_os = "macos") {
            println!("cargo:rustc-link-lib=dylib=c++");
            for framework in ["Foundation", "CoreML", "Accelerate"] {
                println!("cargo:rustc-link-lib=framework={framework}");
            }
        } else {
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }
        return;
    }

    // Falling back to the shared library keeps development working against a
    // downloaded distribution, which is what a study uses. It is not what should
    // be released, and says so.
    println!(
        "cargo:warning=No static sherpa-onnx libraries in {directory}; linking the shared library. A release build should link statically so the sidecar carries no dependency on this machine."
    );
    println!("cargo:rustc-link-lib=dylib=sherpa-onnx-c-api");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{directory}");
}

/// Every `libNAME.a` in the directory, longest name first so that the C API is
/// offered to the linker before the libraries it depends on.
fn archives(directory: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            name.strip_prefix("lib")
                .and_then(|rest| rest.strip_suffix(".a"))
                .map(str::to_string)
        })
        .collect();
    names.sort_by(|a, b| {
        let rank = |name: &str| match name {
            "sherpa-onnx-c-api" => 0,
            "sherpa-onnx-core" => 1,
            name if name.starts_with("onnxruntime") => 9,
            _ => 5,
        };
        rank(a).cmp(&rank(b)).then_with(|| a.cmp(b))
    });
    names
}
