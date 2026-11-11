//! A stub of the GitHub release hosts for the update tests of `tests/cli.rs` and `tests/mcp.rs`.
//! They need a build with the `test-stub` feature (`cargo test -p freemapper-cli --features
//! test-stub`): only there does the program take `FREEMAPPER_UPDATE_TEST_BASE`, count as a release
//! build and send its requests for the GitHub hosts to this stub on 127.0.0.1.

// Each test file uses a part of it.
#![allow(dead_code)]

#[cfg(unix)]
use std::process::Command;

/// The release page of the stub's manifests.
pub const MANIFEST_PATH: &str =
    "/HenryJobst/freemapper/releases/latest/download/freemapper-update.json";

/// A server on 127.0.0.1 in place of the GitHub hosts: each path gets its body, any other a 404.
/// It records the paths asked for, and can wait before each answer.
pub struct UpdateStub {
    pub base: String,
    paths: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    /// Open: the stub answers. Closed: each answer waits until [`UpdateStub::release`].
    gate: std::sync::Arc<(std::sync::Mutex<bool>, std::sync::Condvar)>,
}

impl UpdateStub {
    pub fn start(bodies: Vec<(String, Vec<u8>)>) -> Self {
        Self::answering_after(std::time::Duration::ZERO, bodies)
    }

    pub fn answering_after(delay: std::time::Duration, bodies: Vec<(String, Vec<u8>)>) -> Self {
        Self::serve(delay, true, bodies)
    }

    fn serve(delay: std::time::Duration, open: bool, bodies: Vec<(String, Vec<u8>)>) -> Self {
        use std::io::{BufRead as _, Write as _};
        let gate = std::sync::Arc::new((std::sync::Mutex::new(open), std::sync::Condvar::new()));
        let waits = std::sync::Arc::clone(&gate);
        let bodies: std::collections::HashMap<String, Vec<u8>> = bodies.into_iter().collect();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let paths = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let seen = std::sync::Arc::clone(&paths);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
                let mut request_line = String::new();
                if reader.read_line(&mut request_line).is_err() {
                    continue;
                }
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                        break;
                    }
                }
                let path = request_line
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("")
                    .to_string();
                seen.lock().unwrap().push(path.clone());
                let (open, opened) = &*waits;
                drop(
                    opened
                        .wait_while(open.lock().unwrap(), |open| !*open)
                        .unwrap(),
                );
                std::thread::sleep(delay);
                let (status, body) = match bodies.get(&path) {
                    Some(body) => (200, body.as_slice()),
                    None => (404, &[][..]),
                };
                let head = format!(
                    "HTTP/1.1 {status} Stub\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(body);
            }
        });
        Self { base, paths, gate }
    }

    /// The committed manifest of release 99.0.0 (see [`Self::with_fixture`]), answered only
    /// after [`Self::release`].
    pub fn gated_fixture() -> Self {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/update");
        Self::serve(
            std::time::Duration::ZERO,
            false,
            vec![
                (
                    MANIFEST_PATH.to_string(),
                    std::fs::read(dir.join("manifest-99.json")).unwrap(),
                ),
                (
                    format!("{MANIFEST_PATH}.minisig"),
                    std::fs::read(dir.join("manifest-99.json.minisig")).unwrap(),
                ),
            ],
        )
    }

    /// Lets a gated stub answer, the requests waiting now and all later ones.
    pub fn release(&self) {
        let (open, opened) = &*self.gate;
        *open.lock().unwrap() = true;
        opened.notify_all();
    }

    /// Serves `manifest` and its signature as the latest release.
    pub fn serving(manifest: Vec<u8>, signature: Vec<u8>, more: Vec<(String, Vec<u8>)>) -> Self {
        let mut bodies = vec![
            (MANIFEST_PATH.to_string(), manifest),
            (format!("{MANIFEST_PATH}.minisig"), signature),
        ];
        bodies.extend(more);
        Self::start(bodies)
    }

    /// The committed manifest of release 99.0.0, whose asset hashes match nothing.
    pub fn with_fixture() -> Self {
        Self::with_fixture_after(std::time::Duration::ZERO)
    }

    /// [`Self::with_fixture`], answering each request after `delay`.
    pub fn with_fixture_after(delay: std::time::Duration) -> Self {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/update");
        Self::answering_after(
            delay,
            vec![
                (
                    MANIFEST_PATH.to_string(),
                    std::fs::read(dir.join("manifest-99.json")).unwrap(),
                ),
                (
                    format!("{MANIFEST_PATH}.minisig"),
                    std::fs::read(dir.join("manifest-99.json.minisig")).unwrap(),
                ),
            ],
        )
    }

    pub fn paths(&self) -> Vec<String> {
        self.paths.lock().unwrap().clone()
    }

    pub fn asked_for_an_asset(&self) -> bool {
        self.paths().iter().any(|path| path.ends_with(".gz"))
    }
}

/// A copy of the program in `dir`, for the tests that replace or move it.
pub fn copied_program(dir: &std::path::Path) -> std::path::PathBuf {
    let original = std::path::Path::new(env!("CARGO_BIN_EXE_freemapper"));
    let copy = dir.join(original.file_name().unwrap());
    std::fs::copy(original, &copy).unwrap();
    copy
}

/// A stub serving release 99.0.0 that installs: a manifest signed at run time and a "new
/// program" that is a shell script printing `freemapper 99.0.0`. `None` (and a line saying so)
/// without the `minisign` CLI to sign with.
#[cfg(unix)]
pub fn installable_release(work: &std::path::Path) -> Option<UpdateStub> {
    use std::os::unix::fs::PermissionsExt as _;
    let has_minisign = Command::new("minisign")
        .arg("-v")
        .output()
        .is_ok_and(|output| output.status.success());
    if !has_minisign {
        eprintln!("skipped: no minisign on the PATH to sign the test manifest");
        return None;
    }
    let new = work.join("freemapper-new");
    std::fs::write(&new, "#!/bin/sh\necho \"freemapper 99.0.0\"\n").unwrap();
    std::fs::set_permissions(&new, std::fs::Permissions::from_mode(0o755)).unwrap();
    let gz = Command::new("gzip")
        .args(["-c", "-n"])
        .arg(&new)
        .output()
        .unwrap();
    assert!(gz.status.success());
    let gz = gz.stdout;
    let sha256 = |path: &std::path::Path| -> String {
        let output = Command::new("shasum")
            .args(["-a", "256"])
            .arg(path)
            .output()
            .or_else(|_| Command::new("sha256sum").arg(path).output())
            .unwrap();
        String::from_utf8(output.stdout).unwrap()[..64].to_string()
    };
    std::fs::write(work.join("new.gz"), &gz).unwrap();
    let platform = freemapper_update::current_platform().unwrap();
    let asset_path = format!(
        "/HenryJobst/freemapper/releases/download/v99.0.0/freemapper-v99.0.0-{platform}.gz"
    );
    let manifest = format!(
        r#"{{
  "format": 1,
  "app": "freemapper",
  "version": "99.0.0",
  "released": "2026-11-20",
  "notes": "https://github.com/HenryJobst/freemapper/releases/tag/v99.0.0",
  "min_self_update_from": "0.1.0",
  "assets": {{
    "{platform}": {{
      "url": "https://github.com{asset_path}",
      "size": {size},
      "sha256": "{gz_sha}",
      "exe_sha256": "{exe_sha}"
    }}
  }}
}}
"#,
        size = gz.len(),
        gz_sha = sha256(&work.join("new.gz")),
        exe_sha = sha256(&new),
    );
    let manifest_file = work.join("freemapper-update.json");
    std::fs::write(&manifest_file, &manifest).unwrap();
    let key = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/keys/test.key");
    let signed = Command::new("minisign")
        .arg("-S")
        .arg("-s")
        .arg(&key)
        .arg("-m")
        .arg(&manifest_file)
        .args(["-t", "freemapper test"])
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert!(signed.status.success(), "{signed:?}");
    let signature = std::fs::read(work.join("freemapper-update.json.minisig")).unwrap();
    Some(UpdateStub::serving(
        manifest.into_bytes(),
        signature,
        vec![(asset_path, gz)],
    ))
}
