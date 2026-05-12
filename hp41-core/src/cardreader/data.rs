//! Data card codec — `.card.json` format.
//!
//! No de-facto emulator standard exists for HP-41 data cards (V41 and Free42
//! only cover programs). We define our own JSON format with a clear magic
//! header so future tools can identify it unambiguously.
//!
//! Schema (`format = "hp41-data-v1"`, `version = 1`):
//! ```json
//! {
//!   "format": "hp41-data-v1",
//!   "version": 1,
//!   "registers": ["0", "1.5", "-3.14", ...]   // HpNum values, count = SIZE
//! }
//! ```

use crate::error::HpError;
use crate::num::HpNum;
use serde::{Deserialize, Serialize};

/// Magic header string identifying the format. Future formats bump this tag.
pub const FORMAT_TAG: &str = "hp41-data-v1";

/// Schema version inside the tagged format (additive evolution).
pub const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataCard {
    pub format: String,
    pub version: u32,
    pub registers: Vec<HpNum>,
}

/// Serialize a `DataCard` to pretty JSON bytes (UTF-8) ready for disk write.
pub fn encode_data(card: &DataCard) -> Result<Vec<u8>, HpError> {
    serde_json::to_vec_pretty(card).map_err(|_| HpError::CardData)
}

/// Parse `.card.json` bytes into a `DataCard`, validating magic header.
pub fn decode_data(bytes: &[u8]) -> Result<DataCard, HpError> {
    let card: DataCard = serde_json::from_slice(bytes).map_err(|_| HpError::CardData)?;
    if card.format != FORMAT_TAG {
        return Err(HpError::CardData);
    }
    Ok(card)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn sample_card() -> DataCard {
        DataCard {
            format: FORMAT_TAG.to_string(),
            version: FORMAT_VERSION,
            registers: vec![HpNum::from(0i32), HpNum::from(42i32), HpNum::from(-17i32)],
        }
    }

    #[test]
    fn encode_decode_round_trip() {
        let card = sample_card();
        let bytes = encode_data(&card).unwrap();
        let back = decode_data(&bytes).unwrap();
        assert_eq!(card, back);
    }

    #[test]
    fn decode_rejects_wrong_format_tag() {
        let bad = br#"{"format":"some-other-format","version":1,"registers":[]}"#;
        assert!(matches!(decode_data(bad), Err(HpError::CardData)));
    }

    #[test]
    fn decode_rejects_malformed_json() {
        assert!(matches!(decode_data(b"not json"), Err(HpError::CardData)));
    }

    #[test]
    fn encoded_bytes_contain_magic_tag() {
        let bytes = encode_data(&sample_card()).unwrap();
        let s = std::str::from_utf8(&bytes).unwrap();
        assert!(s.contains(FORMAT_TAG));
    }
}
