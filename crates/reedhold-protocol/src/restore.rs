//! Restore an account from a recovery manifest.

use crate::account::Account;
use reedhold_core::{Error, Result};
use reedhold_recovery::{RecoveryManifest, unseal_seed};

/// Unlock a manifest with `password` and rebuild the live account.
///
/// # Errors
///
/// Returns [`Error::Recovery`] when the password is wrong or the identity
/// inside the manifest does not match the unsealed seed.
pub fn restore_account(
    manifest: &RecoveryManifest,
    password: &[u8],
    device_secret: &[u8; 32],
) -> Result<Account> {
    let seed = unseal_seed(password, &manifest.sealed)?;
    let bundle = seed.unlock()?;
    if bundle.identity != manifest.identity {
        return Err(Error::Recovery("unsealed identity does not match manifest"));
    }
    let device = bundle.devices.device_keys(device_secret)?;
    Ok(Account::new(seed, bundle, device, manifest.network))
}

#[cfg(test)]
mod tests {
    use crate::{create_account, restore_account};
    use reedhold_core::NetworkId;
    use reedhold_event::EventKind;
    use reedhold_recovery::KdfParams;

    #[test]
    fn wipe_and_restore_keeps_identity_and_verifies_old_events() {
        let created =
            create_account(NetworkId::DEV, b"hunter2", &[1_u8; 32], KdfParams::TEST).unwrap();
        let identity = created.account.identity();
        let mut live = created.account;
        let event = live.emit(EventKind::Post, b"hello").unwrap();
        let device_public = live.device_public();
        drop(live);

        let restored = restore_account(&created.manifest, b"hunter2", &[1_u8; 32]).unwrap();
        assert_eq!(restored.identity(), identity);
        let encoded = event.encode().unwrap();
        reedhold_event::SignedEvent::decode_verify(&encoded, NetworkId::DEV, &device_public)
            .unwrap();
    }

    #[test]
    fn password_change_does_not_change_identity() {
        let created = create_account(NetworkId::DEV, b"old", &[2_u8; 32], KdfParams::TEST).unwrap();
        let identity = created.account.identity();
        let rotated = created
            .account
            .change_password(b"new", 2, KdfParams::TEST)
            .unwrap();
        let restored = restore_account(&rotated, b"new", &[2_u8; 32]).unwrap();
        assert_eq!(restored.identity(), identity);
        assert!(restore_account(&rotated, b"old", &[2_u8; 32]).is_err());
    }
}
