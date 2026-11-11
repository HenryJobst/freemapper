//! `freemapper update` (spec 3.4 to 3.6): check for a newer release, install it after asking, go
//! back to the previous version, or store the mode of the daily check.
//!
//! Results go to stdout, the download line to stderr, and errors back to `main`, which prints them
//! as its usual `error:` line with exit code 1.

use std::io::{BufRead as _, IsTerminal as _, Write as _};
use std::path::Path;
use std::process::ExitCode;

use anyhow::anyhow;
use freemapper_update::{
    App, Mode, Outcome, ProcessRunner, UpdateError, Version, check_for_update, exe_path, install,
    load_mode, previous_path, rollback, save_mode,
};

use super::{freemapper_app, mode_from_env, release_build, settings_dir, update_source};
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

/// Runs the command. Exit codes: 0 done or current, 10 `--check` found an update; errors come back
/// for `main` (exit 1).
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
    let outcome = check_for_update(&app, &source, request.version.as_ref()).map_err(failed)?;
    let Outcome::Available {
        version,
        released,
        notes,
        asset,
    } = &outcome
    else {
        // `Current`, and any case a later core adds: nothing to install.
        println!("{}", text("update-current", &[("version", v(app.current))]));
        return Ok(ExitCode::SUCCESS);
    };
    println!(
        "{}",
        text(
            "update-available",
            &[
                ("new", v(version)),
                ("old", v(app.current)),
                ("released", v(released)),
            ]
        )
    );
    println!(
        "{}",
        text(
            "update-details",
            &[("notes", v(notes)), ("size", v(megabytes(asset.size)))]
        )
    );
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
    eprintln!("{}", text("update-installing", &[("version", v(version))]));
    let installed =
        install(&app, &source, &ProcessRunner::new(app.name), &exe, &outcome).map_err(failed)?;
    println!(
        "{}",
        text(
            "update-done",
            &[
                ("old", v(app.current)),
                ("new", v(installed)),
                ("notes", v(notes)),
            ]
        )
    );
    println!("{}", text("update-mcp-restart", &[]));
    Ok(ExitCode::SUCCESS)
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
        Err(UpdateError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => Err(anyhow!(text(
            "update-no-previous",
            &[("path", v(shown(&previous)))]
        ))),
        Err(e) => Err(failed(e)),
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

fn describe(error: &UpdateError) -> String {
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
        assert_eq!(
            describe(&UpdateError::Signature),
            "the update failed: the update signature is not valid"
        );
        assert_eq!(
            shown(Path::new(r"\\?\C:\bin\freemapper.exe")),
            r"C:\bin\freemapper.exe"
        );
        assert_eq!(shown(Path::new("/usr/bin/x")), "/usr/bin/x");
    }
}
