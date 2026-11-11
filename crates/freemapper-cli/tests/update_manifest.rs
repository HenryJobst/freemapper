//! The scripts of the release workflow. `.github/scripts/update-manifest.sh` writes the update
//! manifest: what it writes must be what freemapper-update reads, and it must still verify once
//! signed the way the workflow signs it. `.github/scripts/release-tag.sh` names the release. Both
//! read a temporary Cargo.toml (`CARGO_TOML`), not the real one.
#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use freemapper_update::parse_manifest;
use freemapper_update::platform::PLATFORMS;
use sha2::{Digest as _, Sha256};

const TAG: &str = "v99.0.0";

/// The Cargo.toml the scripts read in these tests: version 99.0.0, no pin.
const CARGO_TOML: &str = "[workspace.package]\nversion = \"99.0.0\"\n";

fn script(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.github/scripts")
        .join(name)
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

/// A dist folder with all five platforms and a `Cargo.toml` with `cargo_toml` in it.
fn packaged_with(cargo_toml: &str) -> tempfile::TempDir {
    let dist = tempfile::tempdir().unwrap();
    for platform in PLATFORMS {
        package(dist.path(), platform);
    }
    std::fs::write(dist.path().join("Cargo.toml"), cargo_toml).unwrap();
    dist
}

fn packaged() -> tempfile::TempDir {
    packaged_with(CARGO_TOML)
}

fn run(dist: &Path, previous: Option<&Path>) -> Output {
    let mut command = Command::new("bash");
    command
        .arg(script("update-manifest.sh"))
        .arg(dist)
        .args(previous)
        .env("CARGO_TOML", dist.join("Cargo.toml"))
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
    let version = "99.0.0";
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

/// A manifest as the previous release would have it, with `min_self_update_from` 0.1.0.
fn previous_manifest(dist: &Path) -> PathBuf {
    let previous = dist.join("previous.json");
    let first = run(dist, None);
    assert!(first.status.success(), "{first:?}");
    let text: String = String::from_utf8(first.stdout)
        .unwrap()
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("\"min_self_update_from\"") {
                "  \"min_self_update_from\": \"0.1.0\",\n".to_string()
            } else {
                format!("{line}\n")
            }
        })
        .collect();
    assert!(text.contains("\"min_self_update_from\": \"0.1.0\""));
    std::fs::write(&previous, text).unwrap();
    previous
}

fn min_self_update_from(output: &Output) -> String {
    assert!(output.status.success(), "{output:?}");
    let manifest = parse_manifest(&output.stdout, "freemapper").unwrap();
    manifest.min_self_update_from.to_string()
}

#[test]
fn min_self_update_from_carries_over_from_the_previous_manifest() {
    let dist = packaged();
    let previous = previous_manifest(dist.path());
    assert_eq!(
        min_self_update_from(&run(dist.path(), Some(&previous))),
        "0.1.0"
    );
}

#[test]
fn the_pin_in_cargo_toml_wins_over_the_carry_over() {
    // Written without spaces around "=", and a key of the same name in another section is not it.
    let dist = packaged_with(&format!(
        "{CARGO_TOML}\n[package.metadata.other]\nmin_self_update_from = \"1.0.0\"\n\n\
         [workspace.metadata.update]\nmin_self_update_from=\"98.0.0\"\n"
    ));
    let previous = previous_manifest(dist.path());
    assert_eq!(min_self_update_from(&run(dist.path(), None)), "98.0.0");
    assert_eq!(
        min_self_update_from(&run(dist.path(), Some(&previous))),
        "98.0.0"
    );
}

#[test]
fn a_pin_above_the_version_is_refused() {
    let dist = packaged_with(&format!(
        "{CARGO_TOML}\n[workspace.metadata.update]\nmin_self_update_from = \"99.1.0\"\n"
    ));
    let output = run(dist.path(), None);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("99.1.0 (from Cargo.toml) is above version 99.0.0"),
        "{output:?}"
    );
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

/// Runs `release-tag.sh` against the test Cargo.toml (version 99.0.0); returns the output and what
/// it wrote to `GITHUB_ENV` and `GITHUB_OUTPUT`.
fn release_tag(dry_run: &str, input: &str, ref_name: &str) -> (Output, String, String) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), CARGO_TOML).unwrap();
    let env_file = dir.path().join("env");
    let output_file = dir.path().join("output");
    let output = Command::new("bash")
        .arg(script("release-tag.sh"))
        .env("CARGO_TOML", dir.path().join("Cargo.toml"))
        .env("DRY_RUN", dry_run)
        .env("TAG_INPUT", input)
        .env("GITHUB_REF_NAME", ref_name)
        .env("GITHUB_ENV", &env_file)
        .env("GITHUB_OUTPUT", &output_file)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let read = |path: &Path| std::fs::read_to_string(path).unwrap_or_default();
    (output, read(&env_file), read(&output_file))
}

#[test]
fn a_dry_run_is_named_after_cargo_toml() {
    let (output, env, outputs) = release_tag("true", "v1.0.0", "my-branch");
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("::warning::Dry run: the tag input v1.0.0 is ignored"),
        "{stdout}"
    );
    assert!(stdout.ends_with("v99.0.0\n"), "{stdout}");
    assert_eq!(env, "TAG=v99.0.0\n");
    assert_eq!(outputs, "tag=v99.0.0\n");
    // No input, or the same one: no warning.
    for input in ["", "v99.0.0"] {
        let (output, env, _) = release_tag("true", input, "my-branch");
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "v99.0.0\n");
        assert_eq!(env, "TAG=v99.0.0\n");
    }
}

#[test]
fn a_release_tag_must_match_cargo_toml() {
    // Pushed tag, dispatch input, and on a push the input is empty.
    for (dry_run, input, ref_name) in [("", "", "v99.0.0"), ("false", "v99.0.0", "main")] {
        let (output, env, _) = release_tag(dry_run, input, ref_name);
        assert!(output.status.success(), "{output:?}");
        assert_eq!(env, "TAG=v99.0.0\n");
    }
    for (input, ref_name) in [("v1.0.0", "v99.0.0"), ("", "main")] {
        let (output, env, outputs) = release_tag("false", input, ref_name);
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("does not match version 99.0.0"),
            "{output:?}"
        );
        assert_eq!((env.as_str(), outputs.as_str()), ("", ""));
    }
}
