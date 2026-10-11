use anyhow::{anyhow, Result};
use argon2::{Algorithm, Argon2, Params, Version};

pub const SALT_LEN: usize = 16;
pub const KEY_LEN: usize = 32;

/// Derives a 256-bit symmetric key from a passphrase and salt using Argon2id.
pub fn derive_key(passphrase: &str, salt: &[u8; SALT_LEN]) -> Result<[u8; KEY_LEN]> {
    // Parameters tuned for interactive performance and strong cryptographic security
    let params = Params::new(
        19 * 1024, // 19 MiB memory
        2,         // 2 iterations
        1,         // 1 degree of parallelism
        Some(KEY_LEN),
    )
    .map_err(|e| anyhow!("Failed to configure Argon2 params: {e}"))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; KEY_LEN];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow!("Argon2 key derivation error: {e}"))?;

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_deterministic() {
        let salt = [7u8; SALT_LEN];
        let key1 = derive_key("my-secret-passphrase", &salt).unwrap();
        let key2 = derive_key("my-secret-passphrase", &salt).unwrap();
        assert_eq!(key1, key2);

        let key3 = derive_key("other-passphrase", &salt).unwrap();
        assert_ne!(key1, key3);
    }
}
