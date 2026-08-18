//! Protocol-level errors that do not depend on a particular crate.

use core::fmt;

/// Recoverable Reedhold failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// Canonical encoding or decoding failed.
    Codec(&'static str),
    /// Identity or device material was malformed.
    Identity(&'static str),
    /// Recovery vault or manifest failed verification.
    Recovery(&'static str),
    /// Event signature or structure failed verification.
    Event(&'static str),
    /// Operating-system entropy was unavailable.
    Entropy,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Codec(reason) => write!(formatter, "codec: {reason}"),
            Self::Identity(reason) => write!(formatter, "identity: {reason}"),
            Self::Recovery(reason) => write!(formatter, "recovery: {reason}"),
            Self::Event(reason) => write!(formatter, "event: {reason}"),
            Self::Entropy => formatter.write_str("system entropy unavailable"),
        }
    }
}

impl std::error::Error for Error {}

/// Result alias for Reedhold crates.
pub type Result<T> = core::result::Result<T, Error>;
