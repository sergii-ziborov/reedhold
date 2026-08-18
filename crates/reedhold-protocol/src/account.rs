//! In-memory account after creation or restore.

use reedhold_core::{IdentityId, NetworkId, Result};
use reedhold_event::{EventKind, SignedEvent, content_id, sign_event};
use reedhold_identity::{DeviceGrant, DeviceKeys, IdentityBundle, MasterSeed};
use reedhold_recovery::{KdfParams, RecoveryManifest, seal_seed};

/// Newly created account plus the first recovery manifest.
pub struct CreatedAccount {
    /// Live account.
    pub account: Account,
    /// Manifest that can be stored on untrusted hosts.
    pub manifest: RecoveryManifest,
}

/// Unlocked account held in process memory.
pub struct Account {
    seed: MasterSeed,
    bundle: IdentityBundle,
    device: DeviceKeys,
    grant: DeviceGrant,
    sequence: u64,
    network: NetworkId,
    manifest: RecoveryManifest,
}

impl Account {
    pub(crate) fn new(
        seed: MasterSeed,
        bundle: IdentityBundle,
        device: DeviceKeys,
        grant: DeviceGrant,
        network: NetworkId,
        manifest: RecoveryManifest,
    ) -> Self {
        Self {
            seed,
            bundle,
            device,
            grant,
            sequence: 0,
            network,
            manifest,
        }
    }

    /// Permanent identity.
    #[must_use]
    pub const fn identity(&self) -> IdentityId {
        self.bundle.identity
    }

    /// Sign a payload as the next device-local event.
    ///
    /// # Errors
    ///
    /// Returns a codec or identity error from the event layer.
    pub fn emit(&mut self, kind: EventKind, payload: &[u8]) -> Result<SignedEvent> {
        self.sequence += 1;
        sign_event(
            self.network,
            self.bundle.identity,
            &self.device,
            self.sequence,
            kind,
            content_id(payload),
        )
    }

    /// Re-seal the same seed under a new password. Identity does not change.
    ///
    /// # Errors
    ///
    /// Returns a recovery error when sealing fails.
    pub fn change_password(
        &mut self,
        password: &[u8],
        params: KdfParams,
    ) -> Result<RecoveryManifest> {
        let epoch = self.manifest.epoch.saturating_add(1);
        let manifest = build_manifest(
            &self.seed,
            self.bundle.identity,
            self.network,
            password,
            epoch,
            params,
        )?;
        self.manifest = manifest.clone();
        Ok(manifest)
    }

    /// Device public key used to verify this account's events.
    #[must_use]
    pub fn device_public(&self) -> [u8; 32] {
        self.device.public_bytes()
    }

    /// Identity-root public key.
    #[must_use]
    pub fn root_public(&self) -> [u8; 32] {
        self.bundle.root.public
    }

    /// Current device grant.
    #[must_use]
    pub const fn grant(&self) -> &DeviceGrant {
        &self.grant
    }

    /// Latest recovery manifest.
    #[must_use]
    pub const fn manifest(&self) -> &RecoveryManifest {
        &self.manifest
    }

    /// Per-device sequence of the last emitted event.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Network this account belongs to.
    #[must_use]
    pub const fn network(&self) -> NetworkId {
        self.network
    }
}

/// Create an account from a fresh seed and a password-protected vault.
///
/// # Errors
///
/// Returns an identity, recovery, or entropy error.
pub fn create_account(
    network: NetworkId,
    password: &[u8],
    device_secret: &[u8; 32],
    params: KdfParams,
) -> Result<CreatedAccount> {
    let seed = MasterSeed::generate()?;
    let bundle = seed.unlock()?;
    let device = bundle.devices.device_keys(device_secret)?;
    let grant = DeviceGrant::issue(&bundle.root, &device, 1);
    let identity = bundle.identity;
    let manifest = build_manifest(&seed, identity, network, password, 1, params)?;
    Ok(CreatedAccount {
        account: Account::new(seed, bundle, device, grant, network, manifest.clone()),
        manifest,
    })
}

fn build_manifest(
    seed: &MasterSeed,
    identity: IdentityId,
    network: NetworkId,
    password: &[u8],
    epoch: u64,
    params: KdfParams,
) -> Result<RecoveryManifest> {
    Ok(RecoveryManifest {
        network,
        identity,
        epoch,
        sealed: seal_seed(password, seed, params)?,
    })
}
