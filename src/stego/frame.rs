use anyhow::{anyhow, Result};
use crc32fast::Hasher;

pub const MAGIC_BYTES: [u8; 4] = [0x53, 0x54, 0x45, 0x47]; // "STEG"
pub const CURRENT_VERSION: u8 = 0x01;
pub const FLAG_ENCRYPTED: u8 = 1 << 0;
pub const FLAG_COMPRESSED: u8 = 1 << 1;

pub const HEADER_PREFIX_LEN: usize = 38; // 4 + 1 + 1 + 16 + 12 + 4
pub const AUTH_TAG_LEN: usize = 16;
pub const CRC_LEN: usize = 4;
pub const TRAILER_LEN: usize = AUTH_TAG_LEN + CRC_LEN; // 20
pub const TOTAL_OVERHEAD: usize = HEADER_PREFIX_LEN + TRAILER_LEN; // 58 bytes

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StegoHeader {
    pub version: u8,
    pub flags: u8,
    pub salt: [u8; 16],
    pub nonce: [u8; 12],
    pub payload_len: u32,
}

impl StegoHeader {
    pub fn is_encrypted(&self) -> bool {
        (self.flags & FLAG_ENCRYPTED) != 0
    }

    pub fn is_compressed(&self) -> bool {
        (self.flags & FLAG_COMPRESSED) != 0
    }

    /// Serializes the header into exactly 38 bytes.
    pub fn to_bytes(&self) -> [u8; HEADER_PREFIX_LEN] {
        let mut bytes = [0u8; HEADER_PREFIX_LEN];
        bytes[0..4].copy_from_slice(&MAGIC_BYTES);
        bytes[4] = self.version;
        bytes[5] = self.flags;
        bytes[6..22].copy_from_slice(&self.salt);
        bytes[22..34].copy_from_slice(&self.nonce);
        bytes[34..38].copy_from_slice(&self.payload_len.to_be_bytes());
        bytes
    }

    /// Parses the 38-byte header prefix.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < HEADER_PREFIX_LEN {
            return Err(anyhow!(
                "Header data too short ({} bytes, expected {})",
                bytes.len(),
                HEADER_PREFIX_LEN
            ));
        }

        if bytes[0..4] != MAGIC_BYTES {
            return Err(anyhow!("No stego signature found: invalid magic bytes"));
        }

        let version = bytes[4];
        if version != CURRENT_VERSION {
            return Err(anyhow!("Unsupported stego protocol version: {version}"));
        }

        let flags = bytes[5];
        let mut salt = [0u8; 16];
        salt.copy_from_slice(&bytes[6..22]);

        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&bytes[22..34]);

        let payload_len = u32::from_be_bytes(bytes[34..38].try_into().unwrap());

        Ok(Self {
            version,
            flags,
            salt,
            nonce,
            payload_len,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StegoFrame {
    pub header: StegoHeader,
    pub payload: Vec<u8>,
    pub auth_tag: [u8; AUTH_TAG_LEN],
    pub crc32: u32,
}

impl StegoFrame {
    pub fn new(
        flags: u8,
        salt: [u8; 16],
        nonce: [u8; 12],
        payload: Vec<u8>,
        auth_tag: [u8; AUTH_TAG_LEN],
    ) -> Self {
        let crc32 = Self::compute_crc(&payload);
        let header = StegoHeader {
            version: CURRENT_VERSION,
            flags,
            salt,
            nonce,
            payload_len: payload.len() as u32,
        };

        Self {
            header,
            payload,
            auth_tag,
            crc32,
        }
    }

    pub fn compute_crc(data: &[u8]) -> u32 {
        let mut hasher = Hasher::new();
        hasher.update(data);
        hasher.finalize()
    }

    pub fn verify_crc(&self) -> bool {
        Self::compute_crc(&self.payload) == self.crc32
    }

    /// Serializes the entire frame:
    /// [ MAGIC (4B) | VERSION (1B) | FLAGS (1B) | SALT (16B) | NONCE (12B) | LEN (4B) | PAYLOAD (NB) | TAG (16B) | CRC (4B) ]
    pub fn to_bytes(&self) -> Vec<u8> {
        let total_size = HEADER_PREFIX_LEN + self.payload.len() + TRAILER_LEN;
        let mut bytes = Vec::with_capacity(total_size);

        bytes.extend_from_slice(&self.header.to_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes.extend_from_slice(&self.auth_tag);
        bytes.extend_from_slice(&self.crc32.to_be_bytes());

        bytes
    }

    /// Parses an entire frame from raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < TOTAL_OVERHEAD {
            return Err(anyhow!(
                "Byte stream too short for stego frame ({} bytes)",
                bytes.len()
            ));
        }

        let header = StegoHeader::from_bytes(&bytes[..HEADER_PREFIX_LEN])?;
        let expected_payload_len = header.payload_len as usize;
        let expected_total = HEADER_PREFIX_LEN + expected_payload_len + TRAILER_LEN;

        if bytes.len() < expected_total {
            return Err(anyhow!(
                "Incomplete frame: expected {} bytes, got {}",
                expected_total,
                bytes.len()
            ));
        }

        let payload_start = HEADER_PREFIX_LEN;
        let payload_end = payload_start + expected_payload_len;
        let payload = bytes[payload_start..payload_end].to_vec();

        let tag_start = payload_end;
        let tag_end = tag_start + AUTH_TAG_LEN;
        let mut auth_tag = [0u8; AUTH_TAG_LEN];
        auth_tag.copy_from_slice(&bytes[tag_start..tag_end]);

        let crc_start = tag_end;
        let crc_end = crc_start + CRC_LEN;
        let crc32 = u32::from_be_bytes(bytes[crc_start..crc_end].try_into().unwrap());

        let frame = Self {
            header,
            payload,
            auth_tag,
            crc32,
        };

        if !frame.verify_crc() {
            return Err(anyhow!("CRC32 checksum mismatch: data corrupted"));
        }

        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_serialization_roundtrip() {
        let salt = [1u8; 16];
        let nonce = [2u8; 12];
        let payload = b"Secret payload test content".to_vec();
        let auth_tag = [3u8; 16];

        let frame = StegoFrame::new(FLAG_ENCRYPTED | FLAG_COMPRESSED, salt, nonce, payload, auth_tag);
        let serialized = frame.to_bytes();

        let deserialized = StegoFrame::from_bytes(&serialized).unwrap();
        assert_eq!(frame, deserialized);
        assert!(deserialized.verify_crc());
    }

    #[test]
    fn test_invalid_magic() {
        let bytes = vec![0u8; TOTAL_OVERHEAD + 10];
        assert!(StegoHeader::from_bytes(&bytes).is_err());
    }
}
