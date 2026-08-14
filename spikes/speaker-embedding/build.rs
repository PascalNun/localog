// The study links the sherpa-onnx C API directly, from a shared-library
// distribution the person running it already has. The location is asked for
// rather than guessed, because a path that happens to be right on one machine is
// a confusing failure on every other one.
fn main() {
    println!("cargo:rerun-if-env-changed=SHERPA_ONNX_LIB");
    let Ok(lib) = std::env::var("SHERPA_ONNX_LIB") else {
        panic!(
            "Set SHERPA_ONNX_LIB to the lib directory of a sherpa-onnx shared \
             distribution, the one holding libsherpa-onnx-c-api."
        );
    };
    println!("cargo:rustc-link-search=native={lib}");
    println!("cargo:rustc-link-lib=dylib=sherpa-onnx-c-api");
    // So the binary finds the library at run time without DYLD_LIBRARY_PATH.
    println!("cargo:rustc-link-arg=-Wl,-rpath,{lib}");
}
