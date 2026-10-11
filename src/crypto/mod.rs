pub mod cipher;
pub mod kdf;

use anyhow::{anyhow, Result};
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use std::io::{Read, Write};

/// Compresses data using Deflate.
pub fn compress_data(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(data)
        .map_err(|e| anyhow!("Compression error: {e}"))?;
    encoder
        .finish()
        .map_err(|e| anyhow!("Compression finish error: {e}"))
}

/// Decompresses Deflate-compressed data.
pub fn decompress_data(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = DeflateDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| anyhow!("Decompression error: {e}"))?;
    Ok(decompressed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_roundtrip() {
        let original = b"The quick brown fox jumps over the lazy dog. Repetition repetition repetition!";
        let compressed = compress_data(original).unwrap();
        let decompressed = decompress_data(&compressed).unwrap();
        assert_eq!(decompressed, original);
    }
}
