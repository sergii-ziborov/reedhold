//! Append-only signed event records.

use reedhold_codec::{Reader, Writer};
use reedhold_core::{Error, Result};

const LOG_TAG: u8 = 0x51;

/// One persisted event: the signed envelope plus the content-addressed body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredEvent {
    /// Canonical signed event bytes.
    pub encoded: Vec<u8>,
    /// Bytes that were hashed into the event payload id.
    pub body: Vec<u8>,
}

/// Encode a log.
///
/// # Errors
///
/// Returns [`Error::Codec`] when a record cannot be written.
pub fn write_log(events: &[StoredEvent]) -> Result<Vec<u8>> {
    let mut writer = Writer::new();
    writer.write_u8(LOG_TAG);
    writer.write_u64(u64::try_from(events.len()).unwrap_or(0));
    for event in events {
        writer.write_bytes(&event.encoded)?;
        writer.write_bytes(&event.body)?;
    }
    Ok(writer.finish())
}

/// Decode a log.
///
/// # Errors
///
/// Returns [`Error::Codec`] when the buffer is truncated or tagged wrong.
pub fn read_log(bytes: &[u8]) -> Result<Vec<StoredEvent>> {
    let mut reader = Reader::new(bytes);
    if reader.read_u8()? != LOG_TAG {
        return Err(Error::Codec("unknown event log tag"));
    }
    let count =
        usize::try_from(reader.read_u64()?).map_err(|_| Error::Codec("event log is too large"))?;
    let mut events = Vec::with_capacity(count);
    for _ in 0..count {
        events.push(StoredEvent {
            encoded: reader.read_bytes()?.to_vec(),
            body: reader.read_bytes()?.to_vec(),
        });
    }
    reader.finish()?;
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::{StoredEvent, read_log, write_log};

    #[test]
    fn log_round_trips() {
        let events = [StoredEvent {
            encoded: b"evt".to_vec(),
            body: b"hi".to_vec(),
        }];
        let encoded = write_log(&events).unwrap();
        assert_eq!(read_log(&encoded).unwrap(), events);
    }
}
