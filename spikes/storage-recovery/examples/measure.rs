use localog_storage_recovery_spike::{ArtifactKind, FaultPoint, RecoveryMode, SpikeStore};
use serde::Serialize;
use std::fs;
use std::time::Instant;

#[derive(Serialize)]
struct Transcript<'a> {
    schema: &'a str,
    language: &'a str,
    segments: Vec<Segment>,
}

#[derive(Serialize)]
struct Segment {
    ordinal: usize,
    start_ms: u64,
    end_ms: u64,
    speaker: &'static str,
    text: String,
}

fn main() {
    let temporary = tempfile::tempdir().expect("temporary storage root");
    let open_started = Instant::now();
    let mut store = SpikeStore::open(temporary.path()).expect("open store");
    let open_elapsed = open_started.elapsed();
    store.create_meeting("meeting-measure").expect("meeting");

    let transcript = Transcript {
        schema: "localog.transcript.v1-spike",
        language: "en",
        segments: (0..7_200)
            .map(|ordinal| Segment {
                ordinal,
                start_ms: ordinal as u64 * 1_500,
                end_ms: ordinal as u64 * 1_500 + 1_400,
                speaker: match ordinal % 3 {
                    0 => "Speaker 1",
                    1 => "Speaker 2",
                    _ => "Speaker 3",
                },
                text: format!(
                    "Synthetic segment {ordinal} records a design review point without real meeting data."
                ),
            })
            .collect(),
    };
    let bytes = serde_json::to_vec(&transcript).expect("serialize transcript");

    let commit_started = Instant::now();
    let revision = store
        .commit_revision(
            "transcript-v1",
            "meeting-measure",
            ArtifactKind::Transcript,
            1,
            &bytes,
            FaultPoint::None,
        )
        .expect("commit revision");
    let commit_elapsed = commit_started.elapsed();

    let read_started = Instant::now();
    let loaded = store.load_verified(&revision).expect("verified read");
    let read_elapsed = read_started.elapsed();
    assert_eq!(loaded.len(), bytes.len());

    let parse_started = Instant::now();
    let parsed: serde_json::Value = serde_json::from_slice(&loaded).expect("parse transcript");
    let parse_elapsed = parse_started.elapsed();
    assert_eq!(parsed["segments"].as_array().map(Vec::len), Some(7_200));

    let recovery_started = Instant::now();
    let startup_report = store
        .recover(RecoveryMode::Startup)
        .expect("startup recovery scan");
    let recovery_elapsed = recovery_started.elapsed();
    assert_eq!(startup_report, Default::default());

    let integrity_started = Instant::now();
    let integrity_report = store
        .recover(RecoveryMode::FullIntegrity)
        .expect("full integrity scan");
    let integrity_elapsed = integrity_started.elapsed();
    assert_eq!(integrity_report, Default::default());

    let database_file = store.root().join("localog.sqlite3");
    let database_bytes = file_size(&database_file);
    let wal_bytes = file_size(&store.root().join("localog.sqlite3-wal"));
    let shm_bytes = file_size(&store.root().join("localog.sqlite3-shm"));
    println!("segments=7200");
    println!("artifact_bytes={}", revision.byte_count);
    println!("database_bytes={database_bytes}");
    println!("wal_bytes={wal_bytes}");
    println!("shm_bytes={shm_bytes}");
    println!(
        "sqlite_set_bytes={}",
        database_bytes + wal_bytes + shm_bytes
    );
    println!("open_schema_ms={:.3}", open_elapsed.as_secs_f64() * 1000.0);
    println!("commit_ms={:.3}", commit_elapsed.as_secs_f64() * 1000.0);
    println!(
        "verified_read_ms={:.3}",
        read_elapsed.as_secs_f64() * 1000.0
    );
    println!("json_parse_ms={:.3}", parse_elapsed.as_secs_f64() * 1000.0);
    println!(
        "startup_recovery_ms={:.3}",
        recovery_elapsed.as_secs_f64() * 1000.0
    );
    println!(
        "full_integrity_ms={:.3}",
        integrity_elapsed.as_secs_f64() * 1000.0
    );
}

fn file_size(path: &std::path::Path) -> u64 {
    fs::metadata(path)
        .map(|metadata| metadata.len())
        .unwrap_or(0)
}
