//! SP1 verifying-key fingerprint helpers (Day 15).
//!
//! `vk.bytes32()` is a 0x-prefixed hex string. On-chain `GlobalConfig.vkey_hash`
//! stores the same 32 bytes. Guest ELF change → new vkey → update constants + notes.

use std::fmt;
use std::path::Path;

use crate::fixtures::happy_fixture_dir;

/// Failed to parse `vk.bytes32()` hex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VkeyError {
    BadPrefix,
    BadLength { got: usize },
    BadHex,
}

impl fmt::Display for VkeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadPrefix => write!(f, "vkey must start with 0x"),
            Self::BadLength { got } => {
                write!(f, "vkey hex must be 64 digits after 0x, got {got}")
            }
            Self::BadHex => write!(f, "vkey contains non-hex characters"),
        }
    }
}

impl std::error::Error for VkeyError {}

/// Parse `0x` + 64 hex digits into `[u8; 32]` (SP1 `HashableKey::bytes32` form).
pub fn parse_vkey_bytes32(s: &str) -> Result<[u8; 32], VkeyError> {
    let s = s.trim();
    let hex = s.strip_prefix("0x").ok_or(VkeyError::BadPrefix)?;
    if hex.len() != 64 {
        return Err(VkeyError::BadLength { got: hex.len() });
    }
    let mut out = [0u8; 32];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let hi = hex_nibble(chunk[0]).ok_or(VkeyError::BadHex)?;
        let lo = hex_nibble(chunk[1]).ok_or(VkeyError::BadHex)?;
        out[i] = (hi << 4) | lo;
    }
    Ok(out)
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Format bytes as a Rust `[u8; 32]` literal for pasting into `VKEY_HASH`.
pub fn format_vkey_rust_array(bytes: &[u8; 32]) -> String {
    let mut lines = String::from("[\n");
    for row in bytes.chunks(8) {
        lines.push_str("    ");
        for (i, b) in row.iter().enumerate() {
            if i > 0 {
                lines.push_str(", ");
            }
            lines.push_str(&format!("0x{b:02x}"));
        }
        lines.push_str(",\n");
    }
    lines.push(']');
    lines
}

/// Read committed happy-path `vkey.bytes32.txt` (no network / setup).
pub fn load_happy_fixture_vkey_bytes() -> Result<[u8; 32], String> {
    load_vkey_bytes_from_file(&happy_fixture_dir().join("vkey.bytes32.txt"))
}

pub fn load_vkey_bytes_from_file(path: &Path) -> Result<[u8; 32], String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("read {}: {e}", path.display()))?;
    parse_vkey_bytes32(&text).map_err(|e| e.to_string())
}

/// Print hex + Rust array for the frozen happy fixture (Day 15 CLI).
pub fn print_happy_fixture_vkey() {
    let bytes = load_happy_fixture_vkey_bytes().expect("happy fixture vkey");
    println!("vk.bytes32() = 0x{}", hex_lower(&bytes));
    println!("VKEY_HASH (Rust):\n{}", format_vkey_rust_array(&bytes));
}

fn hex_lower(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_happy_fixture_vkey() {
        let bytes = load_happy_fixture_vkey_bytes().expect("fixture");
        assert_ne!(bytes, [0u8; 32]);
        assert_eq!(
            hex_lower(&bytes),
            "00b3a15ce4c0ea94e3b0267473c6b7543a80c72d209b2c947f71886c6a5735d7"
        );
        let rendered = format_vkey_rust_array(&bytes);
        assert!(rendered.contains("0x00"));
        assert!(rendered.contains("0xd7"));
    }

    #[test]
    fn parse_rejects_bad_input() {
        assert!(parse_vkey_bytes32("00b3").is_err());
        assert!(parse_vkey_bytes32("0xgg").is_err());
    }
}
