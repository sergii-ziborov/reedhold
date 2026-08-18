//! Hex/string snapshots. Safe to send across FFI.

use reedhold_core::{INVARIANTS, encode_hex};
use reedhold_event::SignedEvent;
use reedhold_protocol::Account;
use serde::Serialize;

/// Public account snapshot. Contains no secrets.
#[derive(Clone, Debug, Serialize)]
pub struct AccountView {
    /// `reedhold:identity:<hex>`.
    pub identity: String,
    /// Device id hex.
    pub device: String,
    /// Device verifying key hex.
    pub device_public: String,
    /// Identity-root verifying key hex.
    pub root_public: String,
    /// Last emitted per-device sequence.
    pub sequence: u64,
    /// Network label.
    pub network: String,
}

impl AccountView {
    pub(crate) fn from_account(account: &Account) -> Self {
        Self {
            identity: account.identity().to_uri(),
            device: account.grant().device.to_hex(),
            device_public: encode_hex(&account.device_public()),
            root_public: encode_hex(&account.root_public()),
            sequence: account.sequence(),
            network: account.network().as_str().to_owned(),
        }
    }

    /// JSON via `blazingly-json`.
    ///
    /// # Errors
    ///
    /// Returns a display string when encoding fails.
    pub fn to_json(&self) -> Result<String, String> {
        blazingly_json::to_string(self).map_err(|error| error.to_string())
    }
}

/// Signed event snapshot.
#[derive(Clone, Debug, Serialize)]
pub struct EventView {
    /// Event kind name.
    pub kind: String,
    /// Payload content id hex.
    pub payload: String,
    /// Per-device sequence.
    pub sequence: u64,
    /// Canonical signed event hex.
    pub event_hex: String,
    /// Bytes that were content-addressed. Hosts persist this beside the event.
    pub body_hex: String,
}

impl EventView {
    pub(crate) fn from_event(event: &SignedEvent, body: &[u8]) -> reedhold_core::Result<Self> {
        Ok(Self {
            kind: event.kind.as_str().to_owned(),
            payload: event.payload.as_digest().to_hex(),
            sequence: event.sequence,
            event_hex: encode_hex(&event.encode()?),
            body_hex: encode_hex(body),
        })
    }
}

/// Recovery manifest snapshot.
#[derive(Clone, Debug, Serialize)]
pub struct ManifestView {
    /// Identity URI.
    pub identity: String,
    /// Vault epoch.
    pub epoch: u64,
    /// Canonical manifest hex. Store this; it is not the password.
    pub manifest_hex: String,
}

/// Protocol invariant names.
#[must_use]
pub fn invariants() -> Vec<String> {
    INVARIANTS.iter().map(|name| (*name).to_owned()).collect()
}
