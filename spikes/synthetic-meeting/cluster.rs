//! Evaluation wrapper, not a second implementation of speaker clustering.
//! Compile directly with rustc; no new crate or dependencies are needed.
#[path = "../../src-tauri/src/clustering.rs"]
mod clustering;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args
        .get(1)
        .expect("usage: cluster <vectors.bin> <speaker-count>");
    let count: usize = args.get(2).expect("a speaker count").parse().unwrap();
    let (indices, vectors) = clustering::read_vectors(std::path::Path::new(path)).unwrap();
    assert!(!vectors.is_empty(), "no usable speaker embeddings");
    let merged = clustering::merge(&vectors);
    let automatic = merged.voices_above(clustering::SAME_VOICE_FLOOR);
    println!("{{\"indices\":{indices:?},\"known\":{:?},\"automatic\":{:?},\"automaticCount\":{automatic},\"floor\":{}}}",
        merged.voices(count), merged.voices(automatic), clustering::SAME_VOICE_FLOOR);
}
