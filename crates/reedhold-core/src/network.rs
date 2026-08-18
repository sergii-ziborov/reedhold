//! Network identifier. Isolates development, test, and production meshes.

use crate::Error;
use core::fmt;

/// Logical network this event or identity belongs to.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NetworkId(&'static str);

impl NetworkId {
    /// Isolated development mesh. Not a production genesis.
    pub const DEV: Self = Self("reedhold-dev-0");

    /// Construct from a static label.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when the label is empty or longer than 64 bytes.
    pub const fn new(label: &'static str) -> Result<Self, Error> {
        if label.is_empty() || label.len() > 64 {
            return Err(Error::Codec("network id must be 1..=64 bytes"));
        }
        Ok(Self(label))
    }

    /// Borrow the label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl fmt::Display for NetworkId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::NetworkId;

    #[test]
    fn rejects_empty() {
        assert!(NetworkId::new("").is_err());
    }
}
