//! V41/Free42-style bare `.raw` byte codec for HP-41 programs.
//!
//! Encodes/decodes a community-style byte stream (no header, no checksum, no
//! trailer) — the bare flavour written by V41, Free42, and `HP41UC /k`,
//! documented at:
//!   - <https://thomasokken.com/free42/importexport.html>
//!   - <https://www.hpmuseum.org/software/41uc.htm>
//!
//! ## Scope (Phase 19 MVP)
//!
//! Encodable subset covers single-byte arithmetic / stack / math / trig ops,
//! synthetic primitives (NULL, GETKEY, hidden-register STO/RCL), alpha-string
//! instructions (LBL/GTO/XEQ "name" with the `F<len>` prefix), and STO/RCL nn.
//! An END marker (`C0 00 0D`) is always appended on encode (D-19).
//!
//! ## Hardware fidelity caveat
//!
//! Single-byte op codes mirror the HP-41 NUT/FOCAL bytes documented by
//! HP41UC and the synthetic table in `ops/mod.rs` (which are themselves
//! marked `[ASSUMED]`). STO/RCL nn use the **internal prefixes 0xE0/0xE1**
//! to avoid colliding with our hidden-register synthetic bytes 0x90-0x92 /
//! 0xB0-0xB2 — these are NOT the hardware bytes. Full V41 binary
//! compatibility (matching `HP41UC` byte-for-byte) is scoped to Phase 22
//! with community test fixtures. Decoder is forgiving: unknown bytes round-
//! trip via `Op::SyntheticByte(b)`.

use crate::error::HpError;
use crate::ops::Op;

/// END marker bytes appended on encode and used as a stop sentinel on decode.
/// Triplet `C0 00 0D` = global END instruction.
const END_MARKER: [u8; 3] = [0xC0, 0x00, 0x0D];

/// Single-byte alpha-string prefix range. `F0..=FF` introduces a 0..=15-char
/// ASCII payload. Used by `LBL`, `GTO`, `XEQ` with a quoted label name.
const ALPHA_PREFIX_BASE: u8 = 0xF0;

/// Two-byte GTO instruction prefix that precedes an alpha-string. `1D Fx ...`.
/// (HP41UC encoding.)
const GTO_ALPHA_PREFIX: u8 = 0x1D;

/// Two-byte XEQ instruction prefix that precedes an alpha-string. `1E Fx ...`.
const XEQ_ALPHA_PREFIX: u8 = 0x1E;

/// Two-byte LBL instruction prefix that precedes an alpha-string. `CF Fx ...`.
const LBL_ALPHA_PREFIX: u8 = 0xCF;

/// Internal prefix for STO nn (n = 00..99). Two-byte form. See module-doc
/// hardware-fidelity caveat — replaced in Phase 22 with HP41UC-faithful bytes.
const STO_REG_PREFIX: u8 = 0xE0;

/// Internal prefix for RCL nn (n = 00..99). Two-byte form.
const RCL_REG_PREFIX: u8 = 0xE1;

/// Encode a sequence of `Op`s to the V41 bare `.raw` byte stream.
///
/// Always appends an END marker so the produced file is well-formed for
/// real-hardware reads. Returns `HpError::CardData` if any op cannot be
/// represented in the Phase 19 subset.
pub fn encode_program(ops: &[Op]) -> Result<Vec<u8>, HpError> {
    let mut out = Vec::with_capacity(ops.len() + END_MARKER.len());
    for op in ops {
        encode_op(op, &mut out)?;
    }
    out.extend_from_slice(&END_MARKER);
    Ok(out)
}

fn encode_op(op: &Op, out: &mut Vec<u8>) -> Result<(), HpError> {
    match op {
        // Single-byte ops mirroring synthetic_byte_to_op (ops/mod.rs).
        Op::Add => out.push(0x40),
        Op::Sub => out.push(0x41),
        Op::Mul => out.push(0x42),
        Op::Div => out.push(0x43),
        Op::Chs => out.push(0x4F),
        Op::Clx => out.push(0x73),
        Op::Rdn => out.push(0x74),
        Op::XySwap => out.push(0x71),
        Op::Sqrt => out.push(0x52),
        Op::Sq => out.push(0x53),
        Op::Log => out.push(0x57),
        Op::Ln => out.push(0x67),
        Op::Recip => out.push(0x60),
        Op::Sin => out.push(0x59),
        Op::Cos => out.push(0x5A),
        Op::Tan => out.push(0x5B),
        Op::Null => out.push(0xCF),
        Op::GetKey => out.push(0xCE),
        Op::StoM => out.push(0xB0),
        Op::StoN => out.push(0xB1),
        Op::StoO => out.push(0xB2),
        Op::RclM => out.push(0x90),
        Op::RclN => out.push(0x91),
        Op::RclO => out.push(0x92),
        Op::Enter => out.push(0x83),
        Op::Lastx => out.push(0x76),
        Op::Rtn => out.push(0xC5),
        // STO nn / RCL nn — internal two-byte form (see module doc on hardware fidelity).
        Op::StoReg(r) if *r <= 99 => {
            out.push(STO_REG_PREFIX);
            out.push(*r);
        }
        Op::RclReg(r) if *r <= 99 => {
            out.push(RCL_REG_PREFIX);
            out.push(*r);
        }
        // Alpha-string program control: LBL/GTO/XEQ "name"
        Op::Lbl(name) => encode_alpha_instruction(LBL_ALPHA_PREFIX, name, out)?,
        Op::Gto(name) => encode_alpha_instruction(GTO_ALPHA_PREFIX, name, out)?,
        Op::Xeq(name) => encode_alpha_instruction(XEQ_ALPHA_PREFIX, name, out)?,
        // Pass-through for synthetic bytes — preserves any byte we couldn't classify
        // on decode through a round-trip.
        Op::SyntheticByte(b) => out.push(*b),
        // Anything else is outside the Phase 19 encoding subset.
        _ => return Err(HpError::CardData),
    }
    Ok(())
}

fn encode_alpha_instruction(prefix: u8, name: &str, out: &mut Vec<u8>) -> Result<(), HpError> {
    let bytes = name.as_bytes();
    if bytes.len() > 15 {
        return Err(HpError::CardData);
    }
    out.push(prefix);
    out.push(ALPHA_PREFIX_BASE | (bytes.len() as u8));
    out.extend_from_slice(bytes);
    Ok(())
}

/// Decode a V41 bare `.raw` byte stream back into a sequence of `Op`s.
///
/// Stops at the first END marker (`C0 00 0D`). Unknown single bytes become
/// `Op::SyntheticByte(b)` so they round-trip through `encode_program`.
pub fn decode_program(bytes: &[u8]) -> Result<Vec<Op>, HpError> {
    let mut ops = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        // Stop at END marker.
        if bytes[i..].starts_with(&END_MARKER) {
            break;
        }
        let b = bytes[i];
        match b {
            0x40 => {
                ops.push(Op::Add);
                i += 1;
            }
            0x41 => {
                ops.push(Op::Sub);
                i += 1;
            }
            0x42 => {
                ops.push(Op::Mul);
                i += 1;
            }
            0x43 => {
                ops.push(Op::Div);
                i += 1;
            }
            0x4F => {
                ops.push(Op::Chs);
                i += 1;
            }
            0x73 => {
                ops.push(Op::Clx);
                i += 1;
            }
            0x74 => {
                ops.push(Op::Rdn);
                i += 1;
            }
            0x71 => {
                ops.push(Op::XySwap);
                i += 1;
            }
            0x52 => {
                ops.push(Op::Sqrt);
                i += 1;
            }
            0x53 => {
                ops.push(Op::Sq);
                i += 1;
            }
            0x57 => {
                ops.push(Op::Log);
                i += 1;
            }
            0x67 => {
                ops.push(Op::Ln);
                i += 1;
            }
            0x60 => {
                ops.push(Op::Recip);
                i += 1;
            }
            0x59 => {
                ops.push(Op::Sin);
                i += 1;
            }
            0x5A => {
                ops.push(Op::Cos);
                i += 1;
            }
            0x5B => {
                ops.push(Op::Tan);
                i += 1;
            }
            0xCE => {
                ops.push(Op::GetKey);
                i += 1;
            }
            0x83 => {
                ops.push(Op::Enter);
                i += 1;
            }
            0x76 => {
                ops.push(Op::Lastx);
                i += 1;
            }
            0xC5 => {
                ops.push(Op::Rtn);
                i += 1;
            }
            0xB0 => {
                ops.push(Op::StoM);
                i += 1;
            }
            0xB1 => {
                ops.push(Op::StoN);
                i += 1;
            }
            0xB2 => {
                ops.push(Op::StoO);
                i += 1;
            }
            0x90 => {
                ops.push(Op::RclM);
                i += 1;
            }
            0x91 => {
                ops.push(Op::RclN);
                i += 1;
            }
            0x92 => {
                ops.push(Op::RclO);
                i += 1;
            }
            STO_REG_PREFIX => {
                let Some(&nn) = bytes.get(i + 1) else {
                    return Err(HpError::CardData);
                };
                if nn > 99 {
                    return Err(HpError::CardData);
                }
                ops.push(Op::StoReg(nn));
                i += 2;
            }
            RCL_REG_PREFIX => {
                let Some(&nn) = bytes.get(i + 1) else {
                    return Err(HpError::CardData);
                };
                if nn > 99 {
                    return Err(HpError::CardData);
                }
                ops.push(Op::RclReg(nn));
                i += 2;
            }
            0xCF => {
                // CF Fx ... → LBL "name". Otherwise standalone NULL (synthetic primitive).
                if let Some(op) = decode_alpha_instruction(bytes, i, |s| Op::Lbl(s.to_string()))? {
                    let len = 2 + (bytes[i + 1] & 0x0F) as usize;
                    ops.push(op);
                    i += len;
                } else {
                    ops.push(Op::Null);
                    i += 1;
                }
            }
            GTO_ALPHA_PREFIX => {
                if let Some(op) = decode_alpha_instruction(bytes, i, |s| Op::Gto(s.to_string()))? {
                    let len = 2 + (bytes[i + 1] & 0x0F) as usize;
                    ops.push(op);
                    i += len;
                } else {
                    return Err(HpError::CardData);
                }
            }
            XEQ_ALPHA_PREFIX => {
                if let Some(op) = decode_alpha_instruction(bytes, i, |s| Op::Xeq(s.to_string()))? {
                    let len = 2 + (bytes[i + 1] & 0x0F) as usize;
                    ops.push(op);
                    i += len;
                } else {
                    return Err(HpError::CardData);
                }
            }
            other => {
                ops.push(Op::SyntheticByte(other));
                i += 1;
            }
        }
    }
    Ok(ops)
}

/// Helper: decode an alpha-string instruction at position `i`, where `bytes[i]`
/// is the instruction prefix and `bytes[i+1]` should be `F<len>`. Returns
/// `Some(Op)` if the alpha form is present, `None` if the byte at `i+1` is
/// not in the `F0..=FF` range.
fn decode_alpha_instruction(
    bytes: &[u8],
    i: usize,
    ctor: impl FnOnce(&str) -> Op,
) -> Result<Option<Op>, HpError> {
    let Some(&len_byte) = bytes.get(i + 1) else {
        return Ok(None);
    };
    if !(ALPHA_PREFIX_BASE..=ALPHA_PREFIX_BASE | 0x0F).contains(&len_byte) {
        return Ok(None);
    }
    let len = (len_byte & 0x0F) as usize;
    let start = i + 2;
    let end = start + len;
    if end > bytes.len() {
        return Err(HpError::CardData);
    }
    let name = std::str::from_utf8(&bytes[start..end]).map_err(|_| HpError::CardData)?;
    Ok(Some(ctor(name)))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn encode_appends_end_marker() {
        let bytes = encode_program(&[Op::Add]).unwrap();
        assert_eq!(bytes, vec![0x40, 0xC0, 0x00, 0x0D]);
    }

    #[test]
    fn encode_empty_program_is_just_end_marker() {
        let bytes = encode_program(&[]).unwrap();
        assert_eq!(bytes, END_MARKER.to_vec());
    }

    #[test]
    fn round_trip_single_byte_ops() {
        let ops = vec![
            Op::Add,
            Op::Sub,
            Op::Mul,
            Op::Div,
            Op::Sin,
            Op::Cos,
            Op::Sqrt,
            Op::Chs,
            Op::Clx,
            Op::Enter,
            Op::Rtn,
            Op::Lastx,
            Op::XySwap,
            Op::Rdn,
            Op::Null,
            Op::GetKey,
            Op::StoM,
            Op::StoO,
            Op::RclM,
            Op::RclO,
        ];
        let bytes = encode_program(&ops).unwrap();
        let back = decode_program(&bytes).unwrap();
        assert_eq!(ops, back);
    }

    #[test]
    fn round_trip_sto_rcl_with_register() {
        let ops = vec![Op::StoReg(0), Op::StoReg(99), Op::RclReg(42), Op::RclReg(0)];
        let bytes = encode_program(&ops).unwrap();
        let back = decode_program(&bytes).unwrap();
        assert_eq!(ops, back);
    }

    #[test]
    fn round_trip_alpha_labels() {
        let ops = vec![
            Op::Lbl("QUAD".to_string()),
            Op::Add,
            Op::Gto("LOOP".to_string()),
            Op::Xeq("FN".to_string()),
        ];
        let bytes = encode_program(&ops).unwrap();
        let back = decode_program(&bytes).unwrap();
        assert_eq!(ops, back);
    }

    #[test]
    fn decode_stops_at_end_marker() {
        // Bytes after END must be ignored.
        let bytes = vec![0x40, 0xC0, 0x00, 0x0D, 0x41, 0x42];
        let ops = decode_program(&bytes).unwrap();
        assert_eq!(ops, vec![Op::Add]);
    }

    #[test]
    fn decode_unknown_byte_becomes_synthetic() {
        // 0xAA is not in our mapping → SyntheticByte(0xAA).
        let bytes = vec![0xAA, 0xC0, 0x00, 0x0D];
        let ops = decode_program(&bytes).unwrap();
        assert_eq!(ops, vec![Op::SyntheticByte(0xAA)]);
    }

    #[test]
    fn encode_label_too_long_returns_card_data() {
        let too_long = "X".repeat(16);
        let err = encode_program(&[Op::Lbl(too_long)]).unwrap_err();
        assert_eq!(err, HpError::CardData);
    }

    #[test]
    fn encode_unsupported_op_returns_card_data() {
        // FmtFix is outside the Phase 19 subset.
        let err = encode_program(&[Op::FmtFix(4)]).unwrap_err();
        assert_eq!(err, HpError::CardData);
    }

    #[test]
    fn decode_truncated_alpha_returns_card_data() {
        // LBL prefix + F4 length, but only 2 bytes of payload follow.
        let bytes = vec![LBL_ALPHA_PREFIX, 0xF4, b'A', b'B'];
        let err = decode_program(&bytes).unwrap_err();
        assert_eq!(err, HpError::CardData);
    }

    #[test]
    fn round_trip_preserves_unknown_bytes_via_synthetic() {
        let original_bytes = vec![0x40, 0xAB, 0x41]; // ADD, unknown, SUB
        let ops = decode_program(&[original_bytes.as_slice(), &END_MARKER].concat()).unwrap();
        assert_eq!(ops, vec![Op::Add, Op::SyntheticByte(0xAB), Op::Sub]);
        let re_encoded = encode_program(&ops).unwrap();
        assert_eq!(&re_encoded[..3], &original_bytes[..]);
    }
}
