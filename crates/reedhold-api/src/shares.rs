//! Threshold recovery for hosts. Shares never include the password.

use crate::session::Session;
use crate::view::ManifestView;
use reedhold_core::{Error, NetworkId, Result, decode32};
use reedhold_protocol::open_seed;
use reedhold_recovery::{KdfParams, SeedShare, combine_seed};
use serde::Serialize;

/// One share in hex, safe to put in a QR or give to a friend.
#[derive(Clone, Debug, Serialize)]
pub struct ShareView {
    /// Shamir x-coordinate.
    pub index: u8,
    /// 32-byte share body, hex.
    pub body_hex: String,
}

impl Session {
    /// Split the unlocked seed. One share cannot restore the account.
    ///
    /// # Errors
    ///
    /// Returns a recovery error when `threshold`/`total` are invalid.
    pub fn split_recovery(&self, threshold: u8, total: u8) -> Result<Vec<ShareView>> {
        let shares = self.account.split_seed(threshold, total)?;
        Ok(shares.iter().map(share_view).collect())
    }
}

/// Combine shares and open a new session under `password`.
///
/// # Errors
///
/// Returns a recovery error when too few shares are supplied.
pub fn session_from_shares(
    shares: &[ShareView],
    threshold: u8,
    password: &str,
    device_secret_hex: &str,
) -> Result<(Session, ManifestView)> {
    let parsed = parse_shares(shares)?;
    let seed = combine_seed(&parsed, threshold)?;
    let device = decode32(device_secret_hex)?;
    let created = open_seed(
        NetworkId::DEV,
        seed,
        password.as_bytes(),
        &device,
        KdfParams::TEST,
    )?;
    let manifest = ManifestView {
        identity: created.account.identity().to_uri(),
        epoch: created.account.manifest().epoch,
        manifest_hex: reedhold_core::encode_hex(&created.account.manifest().encode()?),
    };
    Ok((
        Session {
            account: created.account,
            log: Vec::new(),
        },
        manifest,
    ))
}

fn share_view(share: &SeedShare) -> ShareView {
    ShareView {
        index: share.index,
        body_hex: reedhold_core::encode_hex(&share.body),
    }
}

fn parse_shares(shares: &[ShareView]) -> Result<Vec<SeedShare>> {
    let mut out = Vec::with_capacity(shares.len());
    for share in shares {
        out.push(SeedShare {
            index: share.index,
            body: decode32(&share.body_hex)?,
        });
    }
    if out.is_empty() {
        return Err(Error::Recovery("no shares"));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::session_from_shares;
    use crate::session::Session;

    #[test]
    fn two_shares_restore_the_same_identity() {
        let secret = "33".repeat(32);
        let created = Session::create("old-pw", &secret).unwrap();
        let identity = created.session.view().identity;
        let shares = created.session.split_recovery(2, 3).unwrap();
        let (restored, _) = session_from_shares(
            &[shares[0].clone(), shares[2].clone()],
            2,
            "new-pw",
            &secret,
        )
        .unwrap();
        assert_eq!(restored.view().identity, identity);
        assert!(session_from_shares(&shares[..1], 2, "new-pw", &secret).is_err());
    }
}
