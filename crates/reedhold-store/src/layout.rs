//! Files on disk. Hosts pass the directory; we do not pick a home path.

use crate::eventlog::{StoredEvent, read_log, write_log};
use reedhold_core::{Error, NetworkId, Result};
use reedhold_recovery::RecoveryManifest;
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST_FILE: &str = "manifest.bin";
const EVENTS_FILE: &str = "events.bin";
const CIRCLES_FILE: &str = "circles.bin";

/// A directory that holds one account's sealed manifest and signed events.
#[derive(Clone, Debug)]
pub struct LocalStore {
    root: PathBuf,
}

impl LocalStore {
    /// Bind to `root`. The directory is created on the first write.
    #[must_use]
    pub fn open(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Borrow the directory path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.root
    }

    /// Whether a manifest is already on disk.
    #[must_use]
    pub fn has_manifest(&self) -> bool {
        self.root.join(MANIFEST_FILE).is_file()
    }

    /// Write the sealed recovery manifest.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when the directory or file cannot be written.
    pub fn save_manifest(&self, manifest: &RecoveryManifest) -> Result<()> {
        fs::create_dir_all(&self.root)
            .map_err(|_| Error::Codec("cannot create store directory"))?;
        let bytes = manifest.encode()?;
        fs::write(self.root.join(MANIFEST_FILE), bytes)
            .map_err(|_| Error::Codec("cannot write manifest"))
    }

    /// Read the sealed recovery manifest.
    ///
    /// # Errors
    ///
    /// Returns a recovery error when the file is missing or malformed.
    pub fn load_manifest(&self, network: NetworkId) -> Result<RecoveryManifest> {
        let bytes = fs::read(self.root.join(MANIFEST_FILE))
            .map_err(|_| Error::Recovery("store has no manifest"))?;
        RecoveryManifest::decode(&bytes, network)
    }

    /// Replace the signed event log.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] on I/O failure.
    pub fn save_events(&self, events: &[StoredEvent]) -> Result<()> {
        fs::create_dir_all(&self.root)
            .map_err(|_| Error::Codec("cannot create store directory"))?;
        let bytes = write_log(events)?;
        fs::write(self.root.join(EVENTS_FILE), bytes).map_err(|_| Error::Codec("cannot write log"))
    }

    /// Load the signed event log. Missing file is an empty log.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when the log is truncated.
    pub fn load_events(&self) -> Result<Vec<StoredEvent>> {
        match fs::read(self.root.join(EVENTS_FILE)) {
            Ok(bytes) => read_log(&bytes),
            Err(_) => Ok(Vec::new()),
        }
    }

    /// Write the sealed group book. Missing is fine; the file is optional.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] on I/O failure.
    pub fn save_circles(&self, bytes: &[u8]) -> Result<()> {
        fs::create_dir_all(&self.root)
            .map_err(|_| Error::Codec("cannot create store directory"))?;
        fs::write(self.root.join(CIRCLES_FILE), bytes)
            .map_err(|_| Error::Codec("cannot write group book"))
    }

    /// Load the sealed group book. Missing file is `None`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when the file cannot be read.
    pub fn load_circles(&self) -> Result<Option<Vec<u8>>> {
        match fs::read(self.root.join(CIRCLES_FILE)) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(_) => Ok(None),
        }
    }

    /// Delete the store directory. Used by the reinstall proof.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when removal fails.
    pub fn wipe(&self) -> Result<()> {
        if self.root.exists() {
            fs::remove_dir_all(&self.root).map_err(|_| Error::Codec("cannot wipe store"))?;
        }
        Ok(())
    }
}
