use crate::metadata;
use std::fmt::Display;
use zbus::names::Error as NamesError;

#[derive(Debug)]
pub enum Error {
    Zbus(zbus::Error),
    ZbusNames(NamesError),
    Reqwest(reqwest::Error),
    Notification(notify_rust::error::Error),
    Io(std::io::Error),
    Metadata(metadata::MetadataError),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ZbusNames(e) => write!(f, "DBus names error: {}", e),
            Error::Zbus(e) => write!(f, "DBus error: {}", e),
            Error::Reqwest(e) => write!(f, "HTTP request error: {}", e),
            Error::Notification(e) => write!(f, "Notification error: {}", e),
            Error::Metadata(e) => write!(f, "Metadata error: {}", e),
            Error::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl From<zbus::Error> for Error {
    fn from(err: zbus::Error) -> Self {
        Error::Zbus(err)
    }
}

impl From<metadata::MetadataError> for Error {
    fn from(err: metadata::MetadataError) -> Self {
        Error::Metadata(err)
    }
}

impl From<NamesError> for Error {
    fn from(err: NamesError) -> Self {
        Error::ZbusNames(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Reqwest(err)
    }
}

impl From<notify_rust::error::Error> for Error {
    fn from(err: notify_rust::error::Error) -> Self {
        Error::Notification(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}
