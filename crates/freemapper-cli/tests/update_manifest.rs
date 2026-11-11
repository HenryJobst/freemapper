//! `.github/scripts/update-manifest.sh`, which the release workflow runs to write the update
//! manifest: what it writes must be what freemapper-update reads, and it must still verify once
//! signed the way the workflow signs it.
#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use freemapper_update::parse_manifest;
use freemapper_update::platform::PLATFORMS;
use sha2::{Digest as _, Sha256};

const TAG: &str = "v99.0.0";

fn script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.github/scripts/update-manifest.sh")
}

fn hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// What the Package step of `release.yml` uploads for each platform: the gzip-compressed
/// executable and the SHA-256 of the executable, here with a made-up executable.
fn package(dist: &Path, platform: &str) {
    let exe = dist.join(format!("exe-{platform}"));
    std::fs::write(&exe, format!("freemapper for {platform}\n")).unwrap();
    let gz = Command::new("gzip")
        .args(["-9", "-n", "-c"])
        .arg(&exe)
        .output()
        .unwrap();
    assert!(gz.status.success());
    std::fs::write(
        dist.join(format!("freemapper-{TAG}-{platform}.gz")),
        gz.stdout,
    )
    .unwrap();
    std::fs::write(
        dist.join(format!("freemapper-{TAG}-{platform}.exe.sha256")),
        format!("{}  exe-{platform}\n", hex(&std::fs::read(&exe).unwrap())),
    )
    .unwrap();
    std::fs::remove_file(exe).unwrap();
}

fn packaged() -> tempfile::TempDir {
    let dist = tempfile::tempdir().unwrap();
    for platform in PLATFORMS {
        package(dist.path(), platform);
    }
    dist
}

fn run(dist: &Path, previous: Option<&Path>) -> Output {
    let mut command = Command::new("bash");
    command
        .arg(script())
        .arg(dist)
        .args(previous)
        .env("TAG", TAG)
        .env("GITHUB_REPOSITORY", "HenryJobst/freemapper")
        .env("RELEASED", "2026-11-20")
        .stdin(Stdio::null());
    command.output().unwrap()
}

#[test]
fn the_manifest_parses_and_pins_every_gz() {
    let dist = packaged();
    let output = run(dist.path(), None);
    assert!(output.status.success(), "{output:?}");
    let manifest = parse_manifest(&output.stdout, "freemapper").unwrap();
    let version = env!("CARGO_PKG_VERSION");
    assert_eq!(manifest.version.to_string(), version);
    assert_eq!(manifest.released, "2026-11-20");
    assert_eq!(
        manifest.notes,
        format!("https://github.com/HenryJobst/freemapper/releases/tag/{TAG}")
    );
    // No previous manifest: the first release with the updater.
    assert_eq!(manifest.min_self_update_from.to_string(), version);
    assert_eq!(manifest.assets.len(), 5);
    for platform in PLATFORMS {
        let asset = &manifest.assets[platform];
        let gz =
            std::fs::read(dist.path().join(format!("freemapper-{TAG}-{platform}.gz"))).unwrap();
        assert_eq!(
            asset.url,
            format!(
                "https://github.com/HenryJobst/freemapper/releases/download/{TAG}/freemapper-{TAG}-{platform}.gz"
            )
        );
        assert_eq!(asset.size, gz.len() as u64);
        assert_eq!(asset.sha256, hex(&gz));
        assert_eq!(
            asset.exe_sha256,
            hex(format!("freemapper for {platform}\n").as_bytes())
        );
    }
}

#[test]
fn min_self_update_from_carries_over_from_the_previous_manifest() {
    let dist = packaged();
    let previous = dist.path().join("previous.json");
    let first = run(dist.path(), None);
    let text = String::from_utf8(first.stdout).unwrap().replace(
        &format!(
            "\"min_self_update_from\": \"{}\"",
            env!("CARGO_PKG_VERSION")
        ),
        "\"min_self_update_from\": \"0.1.0\"",
    );
    std::fs::write(&previous, text).unwrap();
    let output = run(dist.path(), Some(&previous));
    assert!(output.status.success(), "{output:?}");
    let manifest = parse_manifest(&output.stdout, "freemapper").unwrap();
    assert_eq!(manifest.min_self_update_from.to_string(), "0.1.0");
}

#[test]
fn a_gz_that_is_not_the_executable_is_refused() {
    let dist = packaged();
    std::fs::write(
        dist.path()
            .join(format!("freemapper-{TAG}-macos-arm64.exe.sha256")),
        format!("{}  other\n", hex(b"another program")),
    )
    .unwrap();
    let output = run(dist.path(), None);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("does not unpack to the executable"),
        "{output:?}"
    );
    // A missing platform is refused too.
    std::fs::remove_file(dist.path().join(format!("freemapper-{TAG}-linux-arm64.gz"))).unwrap();
    let output = run(dist.path(), None);
    assert!(!output.status.success());
}

/// Signs the manifest as the workflow does (`minisign -S`, password on stdin) with the test key
/// and checks it with freemapper-update. Without the `minisign` CLI it says so and passes.
#[test]
fn the_signed_manifest_verifies() {
    let has_minisign = Command::new("minisign")
        .arg("-v")
        .output()
        .is_ok_and(|output| output.status.success());
    if !has_minisign {
        assert!(
            std::env::var_os("FREEMAPPER_TEST_REQUIRE_MINISIGN").is_none(),
            "FREEMAPPER_TEST_REQUIRE_MINISIGN is set, but there is no minisign on the PATH"
        );
        eprintln!("skipped: no minisign on the PATH");
        return;
    }
    let dist = packaged();
    let output = run(dist.path(), None);
    assert!(output.status.success(), "{output:?}");
    let file = dist.path().join("freemapper-update.json");
    std::fs::write(&file, &output.stdout).unwrap();
    let keys = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/keys");
    let signed = Command::new("minisign")
        .arg("-S")
        .arg("-s")
        .arg(keys.join("test.key"))
        .arg("-m")
        .arg(&file)
        .args(["-t", &format!("freemapper-update.json {TAG}")])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(signed.status.success(), "{signed:?}");
    let signature =
        std::fs::read_to_string(dist.path().join("freemapper-update.json.minisig")).unwrap();
    let key = std::fs::read_to_string(keys.join("test.pub")).unwrap();
    freemapper_update::verify(&output.stdout, &signature, &[&key]).unwrap();
}
