use anyhow::{anyhow, Result};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::collections::HashSet;

use super::frame::HEADER_PREFIX_LEN;

pub const DEFAULT_SEED: [u8; 32] = [
    0x53, 0x4f, 0x4e, 0x4f, 0x53, 0x54, 0x45, 0x47, // "SONOSTEG"
    0x5f, 0x44, 0x45, 0x46, 0x41, 0x55, 0x4c, 0x54, // "_DEFAULT"
    0x5f, 0x53, 0x45, 0x45, 0x44, 0x5f, 0x32, 0x30, // "_SEED_20"
    0x32, 0x36, 0x5f, 0x52, 0x55, 0x53, 0x54, 0x21, // "26_RUST!"
];

/// Generates pseudo-random sample indices for LSB dispersal.
pub struct SampleDisperser {
    pub total_samples: usize,
}

impl SampleDisperser {
    pub fn new(total_samples: usize) -> Self {
        Self { total_samples }
    }

    /// Header bits are stored sequentially at the beginning of the audio.
    pub fn header_indices() -> Vec<usize> {
        let bit_count = HEADER_PREFIX_LEN * 8; // 38 * 8 = 304
        (0..bit_count).collect()
    }

    /// Generates non-repeating dispersed sample indices for data bits (payload + trailer).
    /// Uses ChaCha20 PRNG seeded with the cryptographic key or a default seed.
    pub fn generate_data_indices(
        &self,
        seed_key: Option<&[u8; 32]>,
        bit_count: usize,
    ) -> Result<Vec<usize>> {
        let header_bits = HEADER_PREFIX_LEN * 8;
        if self.total_samples < header_bits + bit_count {
            return Err(anyhow!(
                "Audio capacity exceeded: need {} samples, but only {} available",
                header_bits + bit_count,
                self.total_samples
            ));
        }

        let seed = seed_key.unwrap_or(&DEFAULT_SEED);
        let mut rng = ChaCha20Rng::from_seed(*seed);

        let available = self.total_samples - header_bits;
        let mut indices = Vec::with_capacity(bit_count);

        if bit_count * 2 < available {
            // Rejection sampling with HashSet - O(bit_count) average time, minimal memory
            let mut seen = HashSet::with_capacity(bit_count);
            while indices.len() < bit_count {
                use rand::Rng;
                let offset = rng.gen_range(0..available);
                if seen.insert(offset) {
                    indices.push(header_bits + offset);
                }
            }
        } else {
            // For dense embedding, generate all offsets and partial shuffle
            use rand::seq::SliceRandom;
            let mut pool: Vec<usize> = (0..available).collect();
            pool.shuffle(&mut rng);
            for offset in pool.into_iter().take(bit_count) {
                indices.push(header_bits + offset);
            }
        }

        Ok(indices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_indices_length() {
        let indices = SampleDisperser::header_indices();
        assert_eq!(indices.len(), 304);
        assert_eq!(indices[0], 0);
        assert_eq!(indices[303], 303);
    }

    #[test]
    fn test_disperser_deterministic() {
        let disperser = SampleDisperser::new(5000);
        let seed = [9u8; 32];
        let idx1 = disperser.generate_data_indices(Some(&seed), 100).unwrap();
        let idx2 = disperser.generate_data_indices(Some(&seed), 100).unwrap();

        assert_eq!(idx1, idx2);
        assert_eq!(idx1.len(), 100);

        // Verify all indices are >= 304 and unique
        let unique: HashSet<_> = idx1.iter().cloned().collect();
        assert_eq!(unique.len(), 100);
        assert!(idx1.iter().all(|&i| i >= 304 && i < 5000));
    }

    #[test]
    fn test_capacity_exceeded() {
        let disperser = SampleDisperser::new(400);
        let result = disperser.generate_data_indices(None, 200);
        assert!(result.is_err());
    }
}
