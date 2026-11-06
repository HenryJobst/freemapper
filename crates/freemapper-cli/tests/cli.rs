use std::process::Command;

fn freemapper() -> Command {
    Command::new(env!("CARGO_BIN_EXE_freemapper"))
}

#[test]
fn version_flag_prints_name() {
    let output = freemapper().arg("--version").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("freemapper "), "stdout: {stdout}");
}
