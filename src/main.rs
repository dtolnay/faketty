#![deny(unsafe_op_in_unsafe_fn)]
#![allow(
    clippy::empty_enum,
    clippy::let_underscore_untyped,
    clippy::needless_pass_by_value,
    clippy::uninlined_format_args
)]

mod error;

use crate::error::Result;
use clap::{Arg, ArgAction, Command};
use nix::pty::{self, ForkptyResult, Winsize};
use nix::sys::wait::{self, WaitStatus};
use nix::unistd::{self, ForkResult, Pid};
use std::ffi::{CString, OsString};
use std::io::{self, Write};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use std::os::unix::ffi::OsStrExt;
use std::process;

enum Exec {}

fn main() -> ! {
    match try_main() {
        Ok(exec) => match exec {},
        Err(err) => {
            let _ = writeln!(io::stderr(), "faketty: {}", err);
            process::exit(1);
        }
    }
}

const STDIN: BorrowedFd = unsafe { BorrowedFd::borrow_raw(0) };
const STDOUT: BorrowedFd = unsafe { BorrowedFd::borrow_raw(1) };
const STDERR: BorrowedFd = unsafe { BorrowedFd::borrow_raw(2) };

fn try_main() -> Result<Exec> {
    let args = crate::args();
    let new_stdin = STDIN.try_clone_to_owned()?;
    let new_stderr = STDERR.try_clone_to_owned()?;
    let pty1 = unsafe { crate::forkpty() }?;
    if let ForkResult::Parent { child } = pty1.fork_result {
        crate::copyfd(pty1.master.as_fd(), STDOUT);
        crate::copyexit(child);
    }
    let new_stdout = STDOUT.try_clone_to_owned()?;
    let pty2 = unsafe { crate::forkpty() }?;
    if let ForkResult::Parent { child } = pty2.fork_result {
        crate::copyfd(pty2.master.as_fd(), new_stderr.as_fd());
        crate::copyexit(child);
    }
    unistd::dup2(new_stdin.as_raw_fd(), STDIN.as_raw_fd())?;
    unistd::dup2(new_stdout.as_raw_fd(), STDOUT.as_raw_fd())?;
    crate::exec(args)
}

fn app() -> Command {
    let mut app = Command::new("faketty")
        .override_usage("faketty <program> <args...>")
        .help_template("usage: {usage}")
        .arg(
            Arg::new("program")
                .num_args(1..)
                .value_parser(clap::builder::OsStringValueParser::new())
                .required_unless_present_any(["help", "version"])
                .trailing_var_arg(true),
        )
        .arg(Arg::new("help").long("help").action(ArgAction::SetTrue))
        .arg(
            Arg::new("version")
                .long("version")
                .action(ArgAction::SetTrue),
        )
        .disable_help_flag(true)
        .disable_version_flag(true);
    if let Some(version) = option_env!("CARGO_PKG_VERSION") {
        app = app.version(version);
    }
    app
}

fn args() -> Vec<CString> {
    let mut app = app();
    let matches = app.clone().get_matches();
    if matches.get_flag("help") {
        let mut stdout = io::stdout();
        let _ = write!(stdout, "{}", app.render_long_help());
        process::exit(0);
    }
    if matches.get_flag("version") {
        let mut stdout = io::stdout();
        let _ = stdout.write_all(app.render_version().as_bytes());
        process::exit(0);
    }

    matches
        .get_many::<OsString>("program")
        .unwrap()
        .map(|os_string| CString::new(os_string.as_bytes()).unwrap())
        .collect()
}

unsafe fn forkpty() -> Result<ForkptyResult> {
    let winsize = Winsize {
        ws_row: 24,
        ws_col: 80,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let termios = None;
    let result = unsafe { pty::forkpty(&winsize, termios) }?;
    Ok(result)
}

fn exec(args: Vec<CString>) -> Result<Exec> {
    let args: Vec<_> = args.iter().map(CString::as_c_str).collect();
    unistd::execvp(args[0], &args)?;
    unreachable!();
}

fn copyfd(read: BorrowedFd, write: BorrowedFd) {
    const BUF: usize = 4096;
    let mut buf = [0; BUF];
    loop {
        match unistd::read(read.as_raw_fd(), &mut buf) {
            Ok(0) | Err(_) => return,
            Ok(n) => {
                let _ = write_all(write, &buf[..n]);
            }
        }
    }
}

fn write_all(fd: BorrowedFd, mut buf: &[u8]) -> Result<()> {
    while !buf.is_empty() {
        let n = unistd::write(fd.as_raw_fd(), buf)?;
        buf = &buf[n..];
    }
    Ok(())
}

fn copyexit(child: Pid) -> ! {
    let flag = None;
    process::exit(match wait::waitpid(child, flag) {
        Ok(WaitStatus::Exited(_pid, code)) => code,
        _ => 0,
    });
}

#[test]
fn test_cli() {
    app().debug_assert();
}
