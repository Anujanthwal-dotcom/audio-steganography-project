pub mod capacity;
pub mod disperser;
pub mod frame;
pub mod lsb;

use anyhow::{anyhow, Result};
use rand::RngCore;

use crate::crypto::{
    cipher::{decrypt_payload, encrypt_payload, TAG_LEN},
    compress_data, decompress_data,
    kdf::derive_key,
};

pub use capacity::CapacityStats;
pub use disperser::SampleDisperser;
pub use frame::{
    StegoFrame, StegoHeader, FLAG_COMPRESSED, FLAG_ENCRYPTED, HEADER_PREFIX_LEN, TOTAL_OVERHEAD,
    TRAILER_LEN,
};
pub use lsb::{decode_lsb, encode_lsb};

/// Embeds a secret message into audio samples with optional compression and encryption.
pub fn embed_message(
    samples: &mut [i32],
    message: &[u8],
    passphrase: Option<&str>,
    compress: bool,
) -> Result<StegoFrame> {
    if message.is_empty() {
        return Err(anyhow!("Cannot embed an empty message"));
    }

    let mut data_to_embed = message.to_vec();
    let mut flags = 0u8;

    // Optional compression
    if compress {
        if let Ok(compressed) = compress_data(message) {
            // Only use compression if it actually reduces size
            if compressed.len() < message.len() {
                data_to_embed = compressed;
                flags |= FLAG_COMPRESSED;
            }
        }
    }

    let mut salt = [0u8; 16];
    let mut nonce = [0u8; 12];
    let mut auth_tag = [0u8; 16];
    let mut seed_key: Option<[u8; 32]> = None;

    let payload: Vec<u8>;

    if let Some(pwd) = passphrase.filter(|p| !p.trim().is_empty()) {
        flags |= FLAG_ENCRYPTED;
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut nonce);

        let key = derive_key(pwd, &salt)?;
        let (ct, tag) = encrypt_payload(&key, &nonce, &data_to_embed)?;
        payload = ct;
        auth_tag = tag;
        seed_key = Some(key);
    } else {
        payload = data_to_embed;
    }

    let frame = StegoFrame::new(flags, salt, nonce, payload, auth_tag);

    // Verify sample capacity
    let required_bits = (HEADER_PREFIX_LEN + frame.payload.len() + TRAILER_LEN) * 8;
    if samples.len() < required_bits {
        return Err(anyhow!(
            "Insufficient audio capacity: requires {} samples, but carrier only has {} samples",
            required_bits,
            samples.len()
        ));
    }

    // 1. Encode Header prefix (first 304 samples)
    let header_indices = SampleDisperser::header_indices();
    let header_bytes = frame.header.to_bytes();
    encode_lsb(samples, &header_indices, &header_bytes)?;

    // 2. Encode dispersed payload and trailer
    let disperser = SampleDisperser::new(samples.len());
    let data_len = frame.payload.len() + TRAILER_LEN;
    let data_bits_count = data_len * 8;
    let data_indices = disperser.generate_data_indices(seed_key.as_ref(), data_bits_count)?;

    let mut data_bytes = Vec::with_capacity(data_len);
    data_bytes.extend_from_slice(&frame.payload);
    data_bytes.extend_from_slice(&frame.auth_tag);
    data_bytes.extend_from_slice(&frame.crc32.to_be_bytes());

    encode_lsb(samples, &data_indices, &data_bytes)?;

    Ok(frame)
}

/// Detects and parses the stego header prefix from the audio samples.
pub fn detect_stego_header(samples: &[i32]) -> Result<StegoHeader> {
    let header_bits = HEADER_PREFIX_LEN * 8;
    if samples.len() < header_bits {
        return Err(anyhow!(
            "Audio file too small to contain a valid Sonosteg header (needs at least {} samples)",
            header_bits
        ));
    }

    let header_indices = SampleDisperser::header_indices();
    let header_bytes = decode_lsb(samples, &header_indices)?;
    StegoHeader::from_bytes(&header_bytes)
}

/// Extracts and verifies the hidden secret message from audio samples.
pub fn extract_message(samples: &[i32], passphrase: Option<&str>) -> Result<Vec<u8>> {
    let header = detect_stego_header(samples)?;

    let required_total_bits = (HEADER_PREFIX_LEN + header.payload_len as usize + TRAILER_LEN) * 8;
    if samples.len() < required_total_bits {
        return Err(anyhow!(
            "Audio file corrupted: insufficient samples to contain advertised payload"
        ));
    }

    let seed_key = if header.is_encrypted() {
        let pwd = passphrase.ok_or_else(|| {
            anyhow!("This audio contains an encrypted message. Please provide a passphrase.")
        })?;
        if pwd.is_empty() {
            return Err(anyhow!("Passphrase cannot be empty for encrypted stego content"));
        }
        let key = derive_key(pwd, &header.salt)?;
        Some(key)
    } else {
        None
    };

    let disperser = SampleDisperser::new(samples.len());
    let data_len = header.payload_len as usize + TRAILER_LEN;
    let data_bits_count = data_len * 8;
    let data_indices = disperser.generate_data_indices(seed_key.as_ref(), data_bits_count)?;

    let data_bytes = decode_lsb(samples, &data_indices)?;

    let payload = data_bytes[..header.payload_len as usize].to_vec();
    let tag_bytes = &data_bytes[header.payload_len as usize..header.payload_len as usize + TAG_LEN];
    let crc_bytes = &data_bytes[header.payload_len as usize + TAG_LEN..];

    let mut auth_tag = [0u8; TAG_LEN];
    auth_tag.copy_from_slice(tag_bytes);

    let crc32 = u32::from_be_bytes(crc_bytes.try_into().unwrap());

    // Verify CRC32
    if StegoFrame::compute_crc(&payload) != crc32 {
        return Err(anyhow!(
            "Integrity check failed: CRC32 checksum mismatch (audio may be edited or corrupted)"
        ));
    }

    let decrypted_bytes = if header.is_encrypted() {
        let key = seed_key.unwrap();
        decrypt_payload(&key, &header.nonce, &payload, &auth_tag)
            .map_err(|_| anyhow!("Decryption failed: Incorrect passphrase or authentication tag corrupted"))?
    } else {
        payload
    };

    let final_message = if header.is_compressed() {
        decompress_data(&decrypted_bytes)
            .map_err(|e| anyhow!("Failed to decompress extracted payload: {e}"))?
    } else {
        decrypted_bytes
    };

    Ok(final_message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_dummy_samples(count: usize) -> Vec<i32> {
        (0..count).map(|i| ((i as i32 * 12345) % 30000) - 15000).collect()
    }

    #[test]
    fn test_embed_extract_plain_roundtrip() {
        let mut samples = generate_dummy_samples(10000);
        let message = b"Hello, unencrypted stego message!";

        let frame = embed_message(&mut samples, message, None, false).unwrap();
        assert!(!frame.header.is_encrypted());

        let extracted = extract_message(&samples, None).unwrap();
        assert_eq!(extracted, message);
    }

    #[test]
    fn test_embed_extract_encrypted_roundtrip() {
        let mut samples = generate_dummy_samples(10000);
        let message = b"Confidential secret message to transmit securely.";
        let password = "SuperSecretPassword123!";

        let frame = embed_message(&mut samples, message, Some(password), true).unwrap();
        assert!(frame.header.is_encrypted());

        // Extract with correct password
        let extracted = extract_message(&samples, Some(password)).unwrap();
        assert_eq!(extracted, message);

        // Extract with wrong password must fail
        assert!(extract_message(&samples, Some("WrongPassword")).is_err());
    }
}
