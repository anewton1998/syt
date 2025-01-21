#![allow(rustdoc::bare_urls)]
#![doc = include_str!("../README.md")]
use std::{error::Error as StdError, fmt};

pub mod append;
pub mod comments;
pub mod lazy;

/// Error enum for errors thrown by functions in this crate.
#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    YamlError(serde_yml::Error),
    FromUtf8Error(std::string::FromUtf8Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(err) => write!(f, "IO Error: {}", err),
            Error::YamlError(err) => write!(f, "YAML Error: {}", err),
            Error::FromUtf8Error(err) => write!(f, "FromUtf8 Error: {}", err),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::IoError(err) => Some(err),
            Error::YamlError(err) => Some(err),
            Error::FromUtf8Error(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<serde_yml::Error> for Error {
    fn from(err: serde_yml::Error) -> Self {
        Error::YamlError(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Error::FromUtf8Error(err)
    }
}

/// This crate's result type for [Error].
pub type Result<T> = std::result::Result<T, Error>;
