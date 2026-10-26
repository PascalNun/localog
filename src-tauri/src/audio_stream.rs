//! Linux WebKitGTK cannot reliably play media through a custom asset scheme.
//! Serve only repository-resolved WAV files through unguessable loopback URLs.
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use tiny_http::{Header, Method, Response, Server, StatusCode};

pub(crate) struct AudioStream {
    base: String,
    files: Arc<Mutex<HashMap<String, PathBuf>>>,
    stop: Arc<AtomicBool>,
}
impl AudioStream {
    pub fn start() -> std::io::Result<Self> {
        let server = Server::http("127.0.0.1:0").map_err(std::io::Error::other)?;
        let base = format!("http://{}", server.server_addr());
        let files = Arc::new(Mutex::new(HashMap::<String, PathBuf>::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let registered = Arc::clone(&files);
        let stopping = Arc::clone(&stop);
        std::thread::spawn(move || {
            while !stopping.load(Ordering::Relaxed) {
                let Ok(Some(request)) = server.recv_timeout(Duration::from_millis(100)) else {
                    continue;
                };
                // Stream from disk without a recording-sized buffer.
                let path = registered
                    .lock()
                    .ok()
                    .and_then(|files| files.get(request.url()).cloned());
                if !matches!(request.method(), Method::Get | Method::Head) {
                    let _ = request.respond(Response::empty(405));
                    continue;
                }
                let Some(path) = path else {
                    let _ = request.respond(Response::empty(404));
                    continue;
                };
                let Ok(mut file) = File::open(path) else {
                    let _ = request.respond(Response::empty(404));
                    continue;
                };
                let Ok(metadata) = file.metadata() else {
                    let _ = request.respond(Response::empty(404));
                    continue;
                };
                let size = metadata.len();
                let range = request
                    .headers()
                    .iter()
                    .find(|header| header.field.equiv("Range"))
                    .map(|header| header.value.as_str());
                let selected = byte_range(range, size);
                let Some((start, end, partial)) = selected else {
                    let response = Response::empty(416)
                        .with_header(header("Content-Range", &format!("bytes */{size}")));
                    let _ = request.respond(response);
                    continue;
                };
                if file.seek(SeekFrom::Start(start)).is_err() {
                    let _ = request.respond(Response::empty(500));
                    continue;
                }
                let length = end - start + 1;
                let mut headers = vec![
                    header("Content-Type", "audio/wav"),
                    header("Accept-Ranges", "bytes"),
                    header("Cache-Control", "no-store"),
                    header("X-Content-Type-Options", "nosniff"),
                ];
                if partial {
                    headers.push(header(
                        "Content-Range",
                        &format!("bytes {start}-{end}/{size}"),
                    ));
                }
                let response = Response::new(
                    StatusCode(if partial { 206 } else { 200 }),
                    headers,
                    file.take(length),
                    Some(length as usize),
                    None,
                );
                let _ = request.respond(response);
            }
        });
        Ok(Self { base, files, stop })
    }
    pub fn register(&self, path: &Path) -> String {
        let mut files = self.files.lock().expect("audio registry poisoned");
        if let Some((token, _)) = files
            .iter()
            .find(|(_, existing)| existing.as_path() == path)
        {
            return format!("{}{token}", self.base);
        }
        // Two UUIDv7 random portions; neither meeting IDs nor disk paths are exposed.
        let token = format!(
            "/{}{}.wav",
            uuid::Uuid::now_v7().simple(),
            uuid::Uuid::now_v7().simple()
        );
        files.insert(token.clone(), path.to_path_buf());
        format!("{}{token}", self.base)
    }
}
impl Drop for AudioStream {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name, value).expect("static HTTP header")
}
fn byte_range(value: Option<&str>, size: u64) -> Option<(u64, u64, bool)> {
    if size == 0 {
        return None;
    }
    let Some(value) = value else {
        return Some((0, size - 1, false));
    };
    let (first, last) = value.strip_prefix("bytes=")?.split_once('-')?;
    if first.is_empty() {
        let suffix: u64 = last.parse().ok()?;
        return (suffix > 0).then_some((size.saturating_sub(suffix), size - 1, true));
    }
    let start: u64 = first.parse().ok()?;
    let end = if last.is_empty() {
        size - 1
    } else {
        last.parse::<u64>().ok()?.min(size - 1)
    };
    (start < size && end >= start).then_some((start, end, true))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ranges_support_seeking_and_reject_invalid_requests() {
        assert_eq!(byte_range(Some("bytes=4-7"), 10), Some((4, 7, true)));
        assert_eq!(byte_range(Some("bytes=4-"), 10), Some((4, 9, true)));
        assert_eq!(byte_range(Some("bytes=-3"), 10), Some((7, 9, true)));
        for value in [
            "bytes=10-",
            "bytes=7-4",
            "bytes=0-1,4-5",
            "bytes=-0",
            "nope",
        ] {
            assert_eq!(byte_range(Some(value), 10), None);
        }
    }
    #[test]
    fn streams_only_registered_files_with_range_and_head_support() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sample.wav");
        std::fs::write(&path, b"0123456789").unwrap();
        let server = AudioStream::start().unwrap();
        let url = server.register(&path);
        let mut response = ureq::get(&url).header("Range", "bytes=4-7").call().unwrap();
        assert_eq!(response.status(), 206);
        assert_eq!(response.headers()["content-range"], "bytes 4-7/10");
        assert_eq!(response.body_mut().read_to_string().unwrap(), "4567");
        let response = ureq::head(&url).call().unwrap();
        assert_eq!(response.headers()["content-length"], "10");
        assert!(
            ureq::get(format!("{}/sample.wav", server.base))
                .call()
                .is_err()
        );
    }
}
