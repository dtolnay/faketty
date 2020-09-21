use clap::{App, AppSettings, Arg};
use nix::fcntl::{self, FcntlArg, FdFlag};
use nix::pty::{self, ForkptyResult, Winsize};
use nix::sys::wait::{self, WaitStatus};
use nix::unistd::{self, ForkResult, Pid};
use nix::Result;
use std::ffi::CString;
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
    let pty1 = crate::forkpty()?;
    if let ForkResult::Parent { child } = pty1.fork_result {
        crate::copyfd(pty1.master, 1);
        crate::copyexit(child);
    }
    let stdout = crate::dup(1)?;
    let pty2 = crate::forkpty()?;
    if let ForkResult::Parent { child } = pty2.fork_result {
        crate::copyfd(pty2.master, stderr);
        crate::copyexit(child);
    }
    unistd::dup2(stdin, 0)?;
    unistd::dup2(stdout, 1)?;
    crate::exec(args)
}

fn args() -> Vec<CString> {
    let program = Arg::with_name("program").required(true).multiple(true);
    let mut app = App::new("faketty")
        .arg(program)
        .setting(AppSettings::TrailingVarArg);
    if let Some(version) = option_env!("CARGO_PKG_VERSION") {
        app = app.version(version);
    }
    let matches = app.get_matches();
    let args: Vec<_> = matches
        .values_of_os("program")
        .unwrap()
        .map(|os_string| CString::new(os_string.as_bytes()).unwrap())
        .collect();
    if args.is_empty() {
        let _ = writeln!(io::stderr(), "usage: faketty PROGRAM ARGS...");
        process::exit(1);
    };
    args
}

fn dup(fd: RawFd) -> Result<RawFd> {
    let new = unistd::dup(fd)?;
    fcntl::fcntl(new, FcntlArg::F_SETFD(FdFlag::FD_CLOEXEC))?;
    Ok(new)
}

fn forkpty() -> Result<ForkptyResult> {
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
                let _ = unistd::write(write, &buf[..n]);
            }
        }
    }
}

fn copyexit(child: Pid) -> ! {
    let flag = None;
    process::exit(match wait::waitpid(child, flag) {
        Ok(WaitStatus::Exited(_pid, code)) => code,
        _ => 0,
    });
}
