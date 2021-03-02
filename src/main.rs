#![allow(
    clippy::empty_enum,
    clippy::let_underscore_drop,
    clippy::needless_pass_by_value
)]

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

struct PtySize(Option<u16>, Option<u16>);

impl<'a> From<(Option<&'a str>, Option<&'a str>)> for PtySize {
    fn from((width, height): (Option<&'a str>, Option<&'a str>)) -> Self {
        Self(
            width.and_then(|w| w.parse::<u16>().ok()),
            height.and_then(|w| w.parse::<u16>().ok()),
        )
    }
}

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
    let (size, args) = crate::args();

    let size = {
        use terminal_size::{terminal_size, Height, Width};

        let parent_size = terminal_size().map_or((80, 24), |(Width(w), Height(h))| (w, h));
        (
            size.0.unwrap_or(parent_size.0),
            size.1.unwrap_or(parent_size.1),
        )
    };

    let stdin = crate::dup(0)?;
    let stderr = crate::dup(2)?;
    let pty1 = crate::forkpty(size)?;
    if let ForkResult::Parent { child } = pty1.fork_result {
        crate::copyfd(pty1.master, 1);
        crate::copyexit(child);
    }
    let stdout = crate::dup(1)?;
    let pty2 = crate::forkpty(size)?;
    if let ForkResult::Parent { child } = pty2.fork_result {
        crate::copyfd(pty2.master, stderr);
        crate::copyexit(child);
    }
    unistd::dup2(stdin, 0)?;
    unistd::dup2(stdout, 1)?;
    crate::exec(args)
}

fn args() -> (PtySize, Vec<CString>) {
    let mut app = App::new("faketty")
        .usage("faketty [OPTIONS] -- <program> <args...>")
        .arg(
            Arg::with_name("width")
                .short("W")
                .long("width")
                .takes_value(true)
                .help("Sets the width of the pty")
                .validator(|w| {
                    w.parse::<u32>()
                        .map(|_| ())
                        .map_err(|_| "must be a number".to_string())
                }),
        )
        .arg(
            Arg::with_name("height")
                .short("H")
                .long("height")
                .takes_value(true)
                .help("Sets the height of the pty")
                .validator(|w| {
                    w.parse::<u32>()
                        .map(|_| ())
                        .map_err(|_| "must be a number".to_string())
                }),
        )
        .setting(AppSettings::TrailingVarArg)
        .arg(
            Arg::with_name("program")
                .multiple(true)
                .required_unless_one(&["help", "version"]),
        );

    if let Some(version) = option_env!("CARGO_PKG_VERSION") {
        app = app.version(version);
    }

    let matches = app.get_matches();

    (
        PtySize::from((matches.value_of("width"), matches.value_of("height"))),
        matches
            .values_of_os("program")
            .unwrap()
            .map(|os_string| CString::new(os_string.as_bytes()).unwrap())
            .collect(),
    )
}

fn dup(fd: RawFd) -> Result<RawFd> {
    let new = unistd::dup(fd)?;
    fcntl::fcntl(new, FcntlArg::F_SETFD(FdFlag::FD_CLOEXEC))?;
    Ok(new)
}

fn forkpty(size: (u16, u16)) -> Result<ForkptyResult> {
    let winsize = Winsize {
        ws_col: size.0,
        ws_row: size.1,
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
