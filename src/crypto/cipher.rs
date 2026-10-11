use anyhow::{anyhow, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};

pub const NONCE_LEN: usize = 12;
pub const TAG_LEN: usize = 16;

/// Encrypts plaintext using ChaCha20-Poly1305.
/// Returns (ciphertext, tag).
pub fn encrypt_payload(
    key: &[u8; 32],
    nonce: &[u8; NONCE_LEN],
    plaintext: &[u8],
) -> Result<(Vec<u8>, [u8; TAG_LEN])> {
    let cipher_key = Key::from_slice(key);
    let cipher = ChaCha20Poly1305::new(cipher_key);
    let cipher_nonce = Nonce::from_slice(nonce);

    // ChaCha20Poly1305::encrypt appends the 16-byte authentication tag to the ciphertext
    let combined = cipher
        .encrypt(cipher_nonce, plaintext)
        .map_err(|e| anyhow!("Encryption error: {e}"))?;

    if combined.len() < TAG_LEN {
        return Err(anyhow!("Encrypted payload is shorter than tag length"));
    }

    let ct_len = combined.len() - TAG_LEN;
    let ciphertext = combined[..ct_len].to_vec();
    let mut tag = [0u8; TAG_LEN];
    tag.copy_from_slice(&combined[ct_len..]);

    Ok((ciphertext, tag))
}

/// Decrypts ciphertext using ChaCha20-Poly1305 and verifies the authentication tag.
pub fn decrypt_payload(
    key: &[u8; 32],
    nonce: &[u8; NONCE_LEN],
    ciphertext: &[u8],
    tag: &[u8; TAG_LEN],
) -> Result<Vec<u8>> {
    let cipher_key = Key::from_slice(key);
    let cipher = ChaCha20Poly1305::new(cipher_key);
    let cipher_nonce = Nonce::from_slice(nonce);

    let mut combined = Vec::with_capacity(ciphertext.len() + TAG_LEN);
    combined.extend_from_slice(ciphertext);
    combined.extend_from_slice(tag);

    let plaintext = cipher
        .decrypt(cipher_nonce, combined.as_ref())
        .map_err(|e| anyhow!("Decryption failed or invalid authentication tag: {e}"))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let nonce = [7u8; NONCE_LEN];
        let message = b"Hello, secure steganography world!";

        let (ciphertext, tag) = encrypt_payload(&key, &nonce, message).unwrap();
        assert_ne!(ciphertext, message);

        let decrypted = decrypt_payload(&key, &nonce, &ciphertext, &tag).unwrap();
        assert_eq!(decrypted, message);
    }

    #[test]
    fn test_decrypt_tampered_tag() {
        let key = [42u8; 32];
        let nonce = [7u8; NONCE_LEN];
        let message = b"Confidential information";

        let (ciphertext, mut tag) = encrypt_payload(&key, &nonce, message).unwrap();
        tag[0] ^= 0xFF; // Corrupt tag

        assert!(decrypt_payload(&key, &nonce, &ciphertext, &tag).is_err());
    }
}
