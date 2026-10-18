// SPDX-License-Identifier: GPL-3.0-or-later

use zbus::{DBusError, fdo};

use crate::storage;

#[derive(Debug, DBusError)]
#[zbus(prefix = "org.winclip.Daemon1.Error")]
pub enum Error {
    #[zbus(error)]
    ZBus(zbus::Error),
    NotFound(String),
    InvalidArgument(String),
    TooLarge(String),
    Failed(String),
}

impl From<storage::Error> for Error {
    fn from(e: storage::Error) -> Self {
        let message = e.to_string();
        match e {
            storage::Error::NotFound => Self::NotFound(message),
            storage::Error::InvalidArgument(_) => Self::InvalidArgument(message),
            storage::Error::TooLarge { .. } => Self::TooLarge(message),
            _ => Self::Failed(message),
        }
    }
}

impl From<Error> for fdo::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::ZBus(e) => Self::ZBus(e),
            Error::InvalidArgument(message) => Self::InvalidArgs(message),
            Error::NotFound(message) | Error::TooLarge(message) | Error::Failed(message) => {
                Self::Failed(message)
            }
        }
    }
}
