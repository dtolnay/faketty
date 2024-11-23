//! Minimal example of `faketty` without the `clap` dependency

use faketty::run_command;

fn main() -> () {
    let mut args = std::env::args();
    let arg0 = args.next().unwrap_or_default();
    let arg0 = match (env!("CARGO_BIN_NAME").trim(), arg0.trim()) {
        (x, _) if !x.is_empty() => x,
        (_, x) if !x.is_empty() => x,
        _ => "faketty",
    };
    let args: Vec<_> = args
        .map(|x| std::ffi::CString::new(x.as_bytes()).unwrap())
        .collect();
    if args.is_empty() {
        eprintln!("Usage: {arg0} <program> <args...>");
        std::process::exit(1);
    };
    run_command(args).unwrap();
}

/// Tests itself with `cargo run --example`
#[test]
fn run_faketty_min() -> std::io::Result<()> {
    use std::fs::{self, File};
    use std::process::Command;

    let tempdir = scratch::path(env!("CARGO_BIN_NAME"));
    let stdout = tempdir.join("test-stdout");
    let stderr = tempdir.join("test-stderr");

    let cargo_args = format!("run --quiet --example={} --", env!("CARGO_BIN_NAME"));
    let cargo_args: Vec<_> = cargo_args.split_whitespace().collect();
    let status = Command::new("cargo")
        .args(cargo_args)
        .arg("tests/test.sh")
        .stdout(File::create(&stdout)?)
        .stderr(File::create(&stderr)?)
        .status()?;

    assert_eq!(status.code(), Some(6));
    assert_eq!(fs::read(stdout)?, "stdout is tty\r\n".as_bytes());
    assert_eq!(fs::read(stderr)?, "stderr is tty\r\n".as_bytes());
    Ok(())
}
