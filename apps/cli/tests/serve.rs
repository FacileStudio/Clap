//! `clap serve`, through the binary.

#[path = "support/bin.rs"]
mod bin;

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Stdio};
use std::time::Duration;

/// A `clap serve` child, killed when the test ends however it ends.
struct Served(Child);

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn get(base: &str, path: &str, headers: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(base).unwrap();
    stream
        .write_all(format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n{headers}Connection: close\r\n\r\n").as_bytes())
        .unwrap();
    let mut answer = String::new();
    stream.read_to_string(&mut answer).unwrap();
    let status = answer.split_whitespace().nth(1).unwrap().parse().unwrap();
    (status, answer)
}

fn read_until(stream: &mut TcpStream, wanted: &str) {
    let (mut seen, mut buffer) = (String::new(), [0u8; 512]);
    while !seen.contains(wanted) {
        let read = stream.read(&mut buffer).unwrap();
        assert!(read > 0, "the stream closed before `{wanted}`: {seen}");
        seen.push_str(&String::from_utf8_lossy(&buffer[..read]));
    }
}

fn serve() -> (tempfile::TempDir, Served, String) {
    let work = tempfile::tempdir().unwrap();
    assert!(bin::command(work.path(), work.path(), &["new", "talk", "-t", "title"]).status().unwrap().success());
    fs::write(work.path().join("talk/.clap.json"), "{}").unwrap();
    fs::write(work.path().join("talk/assets/clip.bin"), "0123456789").unwrap();
    let mut child = bin::command(work.path(), work.path(), &["serve", "talk", "-p", "0", "--json"]);
    let mut served = Served(child.stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap());
    let mut first = String::new();
    BufReader::new(served.0.stdout.take().unwrap()).read_line(&mut first).unwrap();
    let url = serde_json::from_str::<serde_json::Value>(&first).unwrap()["url"].as_str().unwrap().to_owned();
    (work, served, url.replace("http://localhost", "127.0.0.1"))
}

#[test]
fn serve_shows_the_player_the_project_in_its_frame_and_ranges_of_its_files() {
    let (_work, _served, base) = serve();
    let (status, player) = get(&base, "/", "");
    assert!(status == 200 && player.contains("<title>talk</title>") && player.contains("new EventSource"), "{player}");
    assert!(player.contains("sandbox=\"allow-scripts\"") && player.contains("id=\"scrub\""), "{player}");
    let frame = get(&base, "/frame", "").1;
    assert!(
        frame.contains("Your title") && frame.contains("clap:playing"),
        "the frame holds the project and the runtime"
    );
    let (partial, part) = get(&base, "/assets/clip.bin", "Range: bytes=2-5\r\n");
    assert!(partial == 206 && part.contains("Content-Range: bytes 2-5/10") && part.ends_with("2345"), "{part}");
    assert!(get(&base, "/assets/clip.bin", "").1.ends_with("0123456789"));
    assert_eq!((get(&base, "/.clap.json", "").0, get(&base, "/../Cargo.toml", "").0), (404, 404));
}

#[test]
fn serve_reloads_on_change_and_overlays_errors_on_the_last_good_project() {
    let (work, _served, base) = serve();
    assert!(get(&base, "/frame", "").1.contains("Your title"));
    let mut events = TcpStream::connect(&base).unwrap();
    events.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
    events.write_all(b"GET /__clap/events HTTP/1.1\r\nHost: localhost\r\n\r\n").unwrap();
    read_until(&mut events, ": connected");
    fs::write(work.path().join("talk/clap.md"), "---\nnope: 1\n---\n\n# Broken\n").unwrap();
    read_until(&mut events, "data: reload");
    let broken = get(&base, "/frame", "").1;
    assert!(
        broken.contains("id=\"clap-problems\"") && broken.contains("talk/clap.md:2: unknown key `nope`"),
        "{broken}"
    );
    assert!(broken.contains("Your title"), "the last good project stays under the overlay");
}
