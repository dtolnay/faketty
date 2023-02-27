#![allow(
    clippy::empty_enum,
    clippy::let_underscore_untyped,
    clippy::needless_pass_by_value,
    clippy::uninlined_format_args
)]

use clap::{Arg, ArgAction, Command};
use nix::fcntl::{self, FcntlArg, FdFlag};
use nix::pty::{self, ForkptyResult, Winsize};
use nix::sys::wait::{self, WaitStatus};
use nix::unistd::{self, ForkResult, Pid};
use nix::Result;
use std::ffi::{CString, OsString};
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::RawFd;
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

fn try_main() -> Result<Exec> {
    let args = crate::args();
    let stdin = crate::dup(0)?;
    let stderr = crate::dup(2)?;
    let pty1 = unsafe { crate::forkpty() }?;
    if let ForkResult::Parent { child } = pty1.fork_result {
        crate::copyfd(pty1.master, 1);
        crate::copyexit(child);
    }
    let stdout = crate::dup(1)?;
    let pty2 = unsafe { crate::forkpty() }?;
    if let ForkResult::Parent { child } = pty2.fork_result {
        crate::copyfd(pty2.master, stderr);
        crate::copyexit(child);
    }
    unistd::dup2(stdin, 0)?;
    unistd::dup2(stdout, 1)?;
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

fn dup(fd: RawFd) -> Result<RawFd> {
    let new = unistd::dup(fd)?;
    fcntl::fcntl(new, FcntlArg::F_SETFD(FdFlag::FD_CLOEXEC))?;
    Ok(new)
}

unsafe fn forkpty() -> Result<ForkptyResult> {
    let winsize = Winsize {
        ws_row: 24,
        ws_col: 80,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let termios = None;
    pty::forkpty(&winsize, termios)
}

fn exec(args: Vec<CString>) -> Result<Exec> {
    let args: Vec<_> = args.iter().map(CString::as_c_str).collect();
    unistd::execvp(args[0], &args)?;
    unreachable!();
}

fn copyfd(read: RawFd, write: RawFd) {
    const BUF: usize = 4096;
    let mut buf = [0; BUF];
    loop {
        match unistd::read(read, &mut buf) {
            Ok(0) | Err(_) => return,
            Ok(n) => {
                let _ = write_all(write, &buf[..n]);
            }
        }
    }
}

fn write_all(fd: RawFd, mut buf: &[u8]) -> Result<()> {
    while !buf.is_empty() {
        let n = unistd::write(fd, buf)?;
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
