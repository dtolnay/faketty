use std::fmt::{self, Display};
use std::io;

pub(crate) type Result<T> = std::result::Result<T, Error>;

/// Possible errors for the result of [`run_command`][crate::run_command].
#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum Error {
    Nix(nix::Error),
    Io(io::Error),
}

impl From<nix::Error> for Error {
    fn from(err: nix::Error) -> Self {
        Error::Nix(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Nix(err) => Display::fmt(err, formatter),
            Error::Io(err) => Display::fmt(err, formatter),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Nix(err) => err.source(),
            Error::Io(err) => err.source(),
        }
    }
}
