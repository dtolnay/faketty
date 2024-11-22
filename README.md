faketty
=======

[<img alt="github" src="https://img.shields.io/badge/github-dtolnay/faketty-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/dtolnay/faketty)
[<img alt="crates.io" src="https://img.shields.io/crates/v/faketty.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/faketty)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/dtolnay/faketty/ci.yml?branch=master&style=for-the-badge" height="20">](https://github.com/dtolnay/faketty/actions?query=branch%3Amaster)

A wrapper binary to exec a command in a pty, even if redirecting the output.

This allows logging the stdout and stderr (separately) of a process, without the
output being different from what you'd see in the terminal if you weren't
logging anything.

```console
$ cargo install faketty
```

```console
$ faketty bazel build :target >log/out 2>log/err
          ~~~~~~~~~~~~~~~~~~~ command to run
```

<br>

## Background

When redirecting stdout/err to a pipe or file, a process may detect the output
is no longer going to a tty (because it has no width/height, baud rate, etc) and
may change its behavior accordingly. For example many programs that involve a
progress bar or colored text in a terminal disable those things when the output
is not going to a terminal.

There is a [script(1)] command which makes it possible to redirect a command's
terminal-style output by executing it inside a pseudoterminal (pty) &ndash; a
bidirectional pipe that also has width, height, etc and tricks the process into
thinking it is talking to a real terminal. However, `script` only uses a single
pty, which makes it impossible to demultiplex stdout and stderr to different
places.

[script(1)]: https://man7.org/linux/man-pages/man1/script.1.html

The `faketty` command in this repo is similar to `script --quiet --return
--command '...' /dev/null` except that it preserves distinct stdout and stderr
streams.

<br>

## As a library

`faketty` can be added to the [`dev-dependencies`] of cargo projects,
in which case we can drop the default `clap` crate (for command line argument parsing)
with `--no-default-features`:

```bash
cargo add faketty --dev --no-default-features
```

Note that `faketty::run_command` calls [`exec(3)`], therefore the child process
will replace the current (parent) process.

[`exec(3)`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/exec.html

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this program by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
</sub>
