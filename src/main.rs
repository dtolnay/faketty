use clap::{Arg, ArgAction, Command};
use std::ffi::{CString, OsString};
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::process;

use faketty::run_command;

fn main() -> ! {
    match run_command(crate::args()) {
        Err(err) => {
            let _ = writeln!(io::stderr(), "faketty: {}", err);
            process::exit(1);
        }
        #[allow(unreachable_patterns)]
        Ok(exec) => match exec {},
    }
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

#[test]
fn test_cli() {
    app().debug_assert();
}
