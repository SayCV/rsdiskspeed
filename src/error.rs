
use std::convert;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    /// Simple IO error
    Io(io::Error),
    /// Unexpected error
    Unexpected(String),
}

impl ::std::error::Error for Error {
    // fn description(&self) -> &str {
    //     match *self {
    //         Error::Io(ref e) => e.description(),
    //         Error::Unexpected(_) => "something unexpected",
    //     }
    // }

    fn cause(&self) -> Option<&dyn (::std::error::Error)> {
        match *self {
            Error::Io(ref e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref e) => e.fmt(f),
            Error::Unexpected(ref s) => write!(f, "Unexpected: {}", s),
        }
    }
}

impl convert::From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}
