use std::fmt::Display;
use zbus::names::Error as NamesError;

#[derive(Debug)]
pub enum Error {
    ZbusError(zbus::Error),
    ZbusNamesError(NamesError),
    MetadataError(crate::metadata::MetadataError),
    ReqwestError(reqwest::Error),
    NotificationError(notify_rust::error::Error),
    IoError(std::io::Error),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ZbusNamesError(e) => write!(f, "DBus names error: {}", e),
            Error::ZbusError(e) => write!(f, "DBus error: {}", e),
            Error::MetadataError(e) => write!(f, "Metadata error: {}", e),
            Error::ReqwestError(e) => write!(f, "HTTP request error: {}", e),
            Error::NotificationError(e) => write!(f, "Notification error: {}", e),
            Error::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl From<zbus::Error> for Error {
    fn from(err: zbus::Error) -> Self {
        Error::ZbusError(err)
    }
}

impl From<NamesError> for Error {
    fn from(err: NamesError) -> Self {
        Error::ZbusNamesError(err)
    }
}

impl From<crate::metadata::MetadataError> for Error {
    fn from(err: crate::metadata::MetadataError) -> Self {
        Error::MetadataError(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::ReqwestError(err)
    }
}

impl From<notify_rust::error::Error> for Error {
    fn from(err: notify_rust::error::Error) -> Self {
        Error::NotificationError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}
