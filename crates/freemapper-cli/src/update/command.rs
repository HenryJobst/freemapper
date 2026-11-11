//! `freemapper update` (spec 3.4 to 3.6): check for a newer release, install it after asking, go
//! back to the previous version, or store the mode of the daily check.
//!
//! Results and progress go to stdout, and errors back to `main`, which prints them as its usual
//! `error:` line with exit code 1.

use std::io::{BufRead as _, IsTerminal as _, Write as _};
use std::path::Path;
use std::process::ExitCode;

use anyhow::anyhow;
use freemapper_update::{
    App, Mode, Outcome, ProcessRunner, UpdateError, Version, check_for_update, exe_path, install,
    load_mode, manifest_url, previous_path, rollback, save_mode,
};

use super::{
    HttpUpdateSource, freemapper_app, mode_from_env, release_build, settings_dir, update_source,
};
use crate::text::{text, v};

/// The exit code of `update --check` when a newer version exists.
const UPDATE_AVAILABLE: u8 = 10;

/// What `freemapper update` was asked to do.
pub struct Request {
    /// Only check and report.
    pub check: bool,
    /// Install without asking.
    pub yes: bool,
    /// This release instead of the latest one.
    pub version: Option<Version>,
    /// Swap the previous version back in.
    pub rollback: bool,
    /// Store this mode and do nothing else.
    pub mode: Option<Mode>,
}

/// Runs the command. Exit codes: 0 done or current, 10 `--check` found an update (also one that
/// needs a manual install); errors come back for `main` (exit 1).
pub fn run(request: Request) -> anyhow::Result<ExitCode> {
    // Only a setting, so it works in a build from source too.
    if let Some(mode) = request.mode {
        set_mode(mode)?;
        return Ok(ExitCode::SUCCESS);
    }
    if !release_build() {
        println!("{}", text("update-source-build", &[]));
        return Ok(ExitCode::SUCCESS);
    }
    let app = freemapper_app();
    if request.rollback {
        return roll_back(&app, &running_exe()?);
    }
    let source = update_source();
    let outcome = match check_for_update(&app, &source, request.version.as_ref()) {
        // An update exists, only not by this way (spec 3.3).
        Err(too_old @ UpdateError::TooOldToSelfUpdate { .. }) if request.check => {
            println!("{}", describe(&too_old));
            return Ok(ExitCode::from(UPDATE_AVAILABLE));
        }
        result => result
            .map_err(|e| anyhow!(check_failed(&app, &source, request.version.as_ref(), &e)))?,
    };
    println!("{}", checked(&app, &outcome));
    let Outcome::Available { version, notes, .. } = &outcome else {
        // `Current`, and any case a later core adds: nothing to install.
        return Ok(ExitCode::SUCCESS);
    };
    if request.check {
        return Ok(ExitCode::from(UPDATE_AVAILABLE));
    }
    if !request.yes {
        if !std::io::stdin().is_terminal() {
            return Err(anyhow!(text("update-needs-yes", &[])));
        }
        if !ask()? {
            println!("{}", text("update-not-installed", &[]));
            return Ok(ExitCode::SUCCESS);
        }
    }
    let exe = running_exe()?;
    println!("{}", text("update-installing", &[("version", v(version))]));
    let installed =
        install(&app, &source, &ProcessRunner::new(app.name), &exe, &outcome).map_err(failed)?;
    println!("{}", installed_text(&app, installed, notes));
    println!("{}", text("update-mcp-restart", &[]));
    Ok(ExitCode::SUCCESS)
}

/// What a check found: the newer version with its date, notes and size, or that this one is
/// current.
fn checked(app: &App, outcome: &Outcome) -> String {
    match outcome {
        Outcome::Available {
            version,
            released,
            notes,
            asset,
        } => format!(
            "{}\n{}",
            text(
                "update-available",
                &[
                    ("new", v(version)),
                    ("old", v(app.current)),
                    ("released", v(released)),
                ]
            ),
            text(
                "update-details",
                &[("notes", v(notes)), ("size", v(megabytes(asset.size)))]
            )
        ),
        _ => text("update-current", &[("version", v(app.current))]),
    }
}

/// The line after an install: from which version to which, and the release notes.
fn installed_text(app: &App, installed: Version, notes: &str) -> String {
    text(
        "update-done",
        &[
            ("old", v(app.current)),
            ("new", v(installed)),
            ("notes", v(notes)),
        ],
    )
}

/// The MCP tool `check_update` (spec 3.7) in a release build: what `update --check` prints, or
/// the error for the person. A release that needs a manual install is a result, as for
/// `--check`.
pub fn check_for_mcp() -> Result<String, String> {
    let app = freemapper_app();
    let source = update_source();
    match check_for_update(&app, &source, None) {
        Ok(outcome) => Ok(checked(&app, &outcome)),
        Err(too_old @ UpdateError::TooOldToSelfUpdate { .. }) => Ok(describe(&too_old)),
        Err(e) => Err(check_failed(&app, &source, None, &e)),
    }
}

/// The MCP tool `install_update` (spec 3.7) in a release build, once the person agreed: installs
/// the newest release over the running program, printing nothing. The text for the person, or
/// the error.
pub fn install_for_mcp() -> Result<String, String> {
    let app = freemapper_app();
    let source = update_source();
    let outcome =
        check_for_update(&app, &source, None).map_err(|e| check_failed(&app, &source, None, &e))?;
    let Outcome::Available { notes, .. } = &outcome else {
        return Ok(text("update-current", &[("version", v(app.current))]));
    };
    let exe = exe_path().map_err(|e| describe(&UpdateError::Io(e)))?;
    let installed = install(&app, &source, &ProcessRunner::new(app.name), &exe, &outcome)
        .map_err(|e| describe(&e))?;
    // This server is the one that still runs the old version.
    Ok(format!(
        "{}\n{}",
        installed_text(&app, installed, notes),
        text("update-mcp-restart-now", &[])
    ))
}

/// A failed check in the language of the run. A manifest that is not there (HTTP 404) means a
/// release that cannot install itself; one asked for by `version` may also not exist at all. Only
/// the manifest counts: a missing signature or asset stays a plain failure.
fn check_failed(
    app: &App,
    source: &HttpUpdateSource,
    version: Option<&Version>,
    error: &UpdateError,
) -> String {
    if !matches!(error, UpdateError::Download(_))
        || !source.was_not_found(&manifest_url(app, version))
    {
        return describe(error);
    }
    match version {
        None => text(
            "update-no-manifest",
            &[("page", v(format!("{}/releases/latest", app.repo_url)))],
        ),
        Some(version) => text(
            "update-no-manifest-version",
            &[
                ("version", v(version)),
                ("page", v(format!("{}/releases", app.repo_url))),
            ],
        ),
    }
}

/// Stores `mode` in `settings.toml` and says where. Mentions `FREEMAPPER_UPDATE` when it overrides
/// the stored mode.
fn set_mode(mode: Mode) -> anyhow::Result<()> {
    let dir = settings_dir().ok_or_else(|| anyhow!(text("update-no-config-dir", &[])))?;
    let file = dir.join("settings.toml");
    save_mode(&dir, mode).map_err(|e| {
        anyhow!(text(
            "update-mode-failed",
            &[("path", v(file.display())), ("error", v(e))]
        ))
    })?;
    println!(
        "{}",
        text(
            "update-mode-set",
            &[("mode", v(mode)), ("path", v(file.display()))]
        )
    );
    if let Some(value) = mode_from_env()
        && load_mode(&dir, Some(&value)) != mode
    {
        println!("{}", text("update-mode-env", &[("value", v(value))]));
    }
    Ok(())
}

/// `update --rollback`: the previous version takes the place of the running one.
fn roll_back(app: &App, exe: &Path) -> anyhow::Result<ExitCode> {
    let previous = previous_path(exe);
    match rollback(exe, &ProcessRunner::new(app.name)) {
        Ok(version) => {
            println!(
                "{}",
                text(
                    "update-rolled-back",
                    &[("version", v(version)), ("path", v(shown(&previous)))]
                )
            );
            println!("{}", text("update-mcp-restart", &[]));
            Ok(ExitCode::SUCCESS)
        }
        Err(e) => Err(rollback_error(e, &previous)),
    }
}

/// A failed rollback. A missing file means "no previous version" only when it is the previous
/// version that is missing.
fn rollback_error(error: UpdateError, previous: &Path) -> anyhow::Error {
    match error {
        UpdateError::Io(e)
            if e.kind() == std::io::ErrorKind::NotFound
                && std::fs::symlink_metadata(previous).is_err() =>
        {
            anyhow!(text("update-no-previous", &[("path", v(shown(previous)))]))
        }
        other => failed(other),
    }
}

/// The running executable with links resolved: the file an update replaces.
fn running_exe() -> anyhow::Result<std::path::PathBuf> {
    exe_path().map_err(|e| failed(UpdateError::Io(e)))
}

/// Asks before installing; only `y`, `yes` (and the German `j`, `ja`) install.
fn ask() -> anyhow::Result<bool> {
    print!("{} ", text("update-ask", &[]));
    std::io::stdout().flush()?;
    let mut answer = String::new();
    std::io::stdin().lock().read_line(&mut answer)?;
    Ok(is_yes(&answer))
}

fn is_yes(answer: &str) -> bool {
    matches!(
        answer.trim().to_lowercase().as_str(),
        "y" | "yes" | "j" | "ja"
    )
}

/// An update error in the language of the run. The cases the user can act on have their own
/// text; any other keeps the core's English detail inside a translated frame.
fn failed(error: UpdateError) -> anyhow::Error {
    anyhow!(describe(&error))
}

pub(super) fn describe(error: &UpdateError) -> String {
    match error {
        UpdateError::NotWritable(folder) => {
            text("update-not-writable", &[("path", v(shown(folder)))])
        }
        UpdateError::TooOldToSelfUpdate {
            needed,
            version,
            notes,
        } => text(
            "update-too-old",
            &[
                ("new", v(version)),
                ("old", v(freemapper_app().current)),
                ("needed", v(needed)),
                ("notes", v(notes)),
            ],
        ),
        UpdateError::Busy(lock) => text("update-busy", &[("path", v(shown(lock)))]),
        UpdateError::Signature | UpdateError::UnknownKey => {
            text("update-bad-signature", &[("error", v(error))])
        }
        UpdateError::NoAssetForPlatform(platform) => {
            text("update-no-asset", &[("platform", v(platform))])
        }
        other => text("update-failed", &[("error", v(other))]),
    }
}

/// `bytes` in MB (2^20 bytes, as the disk space messages count) with one decimal, written with
/// the decimal separator of the language.
fn megabytes(bytes: u64) -> String {
    format!("{:.1}", bytes as f64 / f64::from(1u32 << 20))
        .replace('.', &text("update-decimal-separator", &[]))
}

/// A path for a message: on Windows without the `\\?\` prefix that resolving the executable adds.
fn shown(path: &Path) -> String {
    let text = path.display().to_string();
    match text.strip_prefix(r"\\?\") {
        Some(plain) => plain.to_string(),
        None => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_yes_installs() {
        for yes in ["y\n", "Y", " yes \r\n", "j", "Ja"] {
            assert!(is_yes(yes), "{yes:?}");
        }
        for no in ["", "\n", "n", "no", "nein", "yess", "maybe"] {
            assert!(!is_yes(no), "{no:?}");
        }
    }

    #[test]
    fn sizes_are_megabytes_with_one_decimal() {
        // The tests run in English (the default language of a unit test).
        assert_eq!(megabytes(12_345_678), "11.8");
        assert_eq!(megabytes(0), "0.0");
        assert_eq!(megabytes(1 << 20), "1.0");
    }

    #[test]
    fn errors_the_user_can_act_on_have_their_own_text() {
        let folder = Path::new("/opt/freemapper");
        assert_eq!(
            describe(&UpdateError::NotWritable(folder.to_path_buf())),
            text("update-not-writable", &[("path", v(folder.display()))])
        );
        let too_old = describe(&UpdateError::TooOldToSelfUpdate {
            needed: "0.12.0".parse().unwrap(),
            version: "0.13.0".parse().unwrap(),
            notes: "https://github.com/HenryJobst/freemapper/releases/tag/v0.13.0".into(),
        });
        assert!(
            too_old.contains("0.13.0") && too_old.contains("0.12.0"),
            "{too_old}"
        );
        assert!(too_old.contains("releases/tag/v0.13.0"), "{too_old}");
        assert!(too_old.contains("the self-update of"), "{too_old}");
        for unverified in [UpdateError::Signature, UpdateError::UnknownKey] {
            let detail = unverified.to_string();
            assert_eq!(
                describe(&unverified),
                text("update-bad-signature", &[("error", v(detail))])
            );
        }
        assert_eq!(
            describe(&UpdateError::Signature),
            "the release could not be verified (the update signature is not valid); nothing was installed"
        );
        assert_eq!(
            describe(&UpdateError::NoAssetForPlatform("linux-riscv".into())),
            text("update-no-asset", &[("platform", v("linux-riscv"))])
        );
        let lock = Path::new("/opt/freemapper/freemapper.update-lock");
        assert_eq!(
            describe(&UpdateError::Busy(lock.to_path_buf())),
            text("update-busy", &[("path", v(lock.display()))])
        );
        assert_eq!(
            shown(Path::new(r"\\?\C:\bin\freemapper.exe")),
            r"C:\bin\freemapper.exe"
        );
        assert_eq!(shown(Path::new("/usr/bin/x")), "/usr/bin/x");
    }

    #[test]
    fn a_missing_file_is_no_previous_only_when_previous_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let previous = dir.path().join("freemapper.previous");
        let not_found = || UpdateError::Io(std::io::ErrorKind::NotFound.into());
        let no_previous = text("update-no-previous", &[("path", v(previous.display()))]);
        assert_eq!(
            rollback_error(not_found(), &previous).to_string(),
            no_previous
        );
        // Another file that is missing (the previous is there) is not "no previous".
        std::fs::write(&previous, "x").unwrap();
        let other = rollback_error(not_found(), &previous).to_string();
        assert_ne!(other, no_previous);
        assert!(other.starts_with("the update failed"), "{other}");
    }
}
