//! Keeps the program under test away from the configuration of whoever runs the tests.

use std::path::Path;
use std::process::Command;

/// `command` with a home folder of the tests. freemapper reads the person's adapter folder and
/// their `allowed-hosts.toml` from the configuration folder of the system, and a run may
/// rewrite that file (it drops hosts a state names itself by now). A test must never see or
/// touch the real ones: adapters in the person's folder would also change what the tests
/// assert, the hosts of the area page for one.
///
/// The home folder is one for all tests, under the target folder. Nothing a test writes there
/// is read by another test: tests that write a configuration bring a folder of their own.
/// On Windows the configuration folder does not follow the environment; there only the adapter
/// folder is redirected.
pub fn isolated(mut command: Command) -> Command {
    let home = Path::new(env!("CARGO_TARGET_TMPDIR")).join("home");
    std::fs::create_dir_all(&home).unwrap();
    command
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_CACHE_HOME", home.join("cache"))
        .env("FREEMAPPER_ADAPTER_DIR", home.join("no-adapters"));
    command
}
