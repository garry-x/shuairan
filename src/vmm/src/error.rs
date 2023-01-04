// Copyright 2022 Garry Xu
// SPDX-License-Identifier: Apache-2.0

use utils::json;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors   
#[derive(Debug, PartialEq)]
pub enum Error {
        /// The required configuration is missing.
    MissingConfig(String),
    /// The configuration provided is illegal.
    IllegalConfig(String),
    /// Errors generated when paring configurations from JSON strings.
    ParsingError(String),
    /// Errors generated when doing file operations.
    IOError(String),
    /// Error rasied by calling kvm ioctls, its format: (errno, info string).  
    IoctlError(i32, String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;

        match self {
            MissingConfig(s) => write!(
                f, 
                "The required configuration for {} is missing.", 
                s
            ),
            IllegalConfig(s) => write!(
                f, 
                "The given configuration for {} is illegal.", 
                s
            ),
            ParsingError(s) => write!(f, "{}", s),
            IOError(s) => write!(f, "I/O error, error={}", s),
            IoctlError(errno, msg) => {
                write!(f, "Failed kvm ioctl, error=({}, {})", errno, msg)
            },
        }
    }
}

impl From<kvm_ioctls::Error> for Error {
    fn from(e: kvm_ioctls::Error) -> Self {
        Error::IoctlError(e.errno(), e.to_string())
    }
}

impl From<json::Error> for Error {
    // Convert a json::Error to config::Error
    fn from(e: json::Error) -> Self {
        use json::Error::*;
        match e {
            ParsingError(_) => Error::ParsingError(e.to_string()),
            IOError(s) => Error::IOError(s),
        }
    }
}
