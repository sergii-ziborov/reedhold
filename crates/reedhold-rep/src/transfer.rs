//! Reputation cannot be bought or sent.

use reedhold_core::{Error, IdentityId, Result};

/// Always fails. Reputation is not a token.
///
/// # Errors
///
/// Returns [`Error::Reputation`] for every call.
pub fn transfer(_from: IdentityId, _to: IdentityId, _amount: u32) -> Result<()> {
    Err(Error::Reputation("reputation is not transferable"))
}

#[cfg(test)]
mod tests {
    use super::transfer;
    use reedhold_core::{Digest32, IdentityId};

    #[test]
    fn cannot_send_reputation() {
        let a = IdentityId::from_digest(Digest32::from_bytes([1; 32]));
        let b = IdentityId::from_digest(Digest32::from_bytes([2; 32]));
        assert!(transfer(a, b, 100).is_err());
    }
}
