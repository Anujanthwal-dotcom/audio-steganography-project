use anyhow::{anyhow, Result};

/// Encodes a stream of bytes into audio samples by replacing the least significant bit (LSB).
/// Each byte requires 8 sample indices (1 bit per sample, MSB first).
pub fn encode_lsb(samples: &mut [i32], indices: &[usize], data: &[u8]) -> Result<()> {
    let required_bits = data.len() * 8;
    if indices.len() < required_bits {
        return Err(anyhow!(
            "Not enough sample indices for encoding (need {}, got {})",
            required_bits,
            indices.len()
        ));
    }

    let mut bit_cursor = 0;
    for &byte in data {
        for bit_pos in (0..8).rev() {
            let bit = ((byte >> bit_pos) & 1) as i32;
            let sample_idx = indices[bit_cursor];
            if sample_idx >= samples.len() {
                return Err(anyhow!("Sample index out of bounds: {sample_idx}"));
            }
            // Clear bit 0 and insert the new bit
            samples[sample_idx] = (samples[sample_idx] & !1) | bit;
            bit_cursor += 1;
        }
    }

    Ok(())
}

/// Decodes bytes from audio samples by reading the least significant bit (LSB).
/// Expects indices.len() to be a multiple of 8.
pub fn decode_lsb(samples: &[i32], indices: &[usize]) -> Result<Vec<u8>> {
    if indices.len() % 8 != 0 {
        return Err(anyhow!(
            "Indices length must be a multiple of 8 to decode full bytes (got {})",
            indices.len()
        ));
    }

    let num_bytes = indices.len() / 8;
    let mut result = Vec::with_capacity(num_bytes);

    let mut bit_cursor = 0;
    for _ in 0..num_bytes {
        let mut byte: u8 = 0;
        for _ in 0..8 {
            let sample_idx = indices[bit_cursor];
            if sample_idx >= samples.len() {
                return Err(anyhow!("Sample index out of bounds: {sample_idx}"));
            }
            let bit = (samples[sample_idx] & 1) as u8;
            byte = (byte << 1) | bit;
            bit_cursor += 1;
        }
        result.push(byte);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsb_roundtrip() {
        let mut samples = vec![100, -200, 305, -407, 50, 61, 72, 83, -95, 106, 117, 128, 139, 140, 151, 162];
        let indices: Vec<usize> = (0..16).collect();
        let original_data = vec![0xAB, 0xCD];

        encode_lsb(&mut samples, &indices, &original_data).unwrap();
        let decoded = decode_lsb(&samples, &indices).unwrap();

        assert_eq!(decoded, original_data);
    }

    #[test]
    fn test_lsb_negative_numbers() {
        let mut samples = vec![-1, -2, -3, -4, -5, -6, -7, -8];
        let indices: Vec<usize> = (0..8).collect();
        let data = vec![0b10101010];

        encode_lsb(&mut samples, &indices, &data).unwrap();
        let decoded = decode_lsb(&samples, &indices).unwrap();

        assert_eq!(decoded, data);
    }
}
