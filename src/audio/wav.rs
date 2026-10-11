use anyhow::{anyhow, Context, Result};
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use std::fs;
use std::path::Path;

use super::metadata::AudioMetadata;

/// Represents in-memory WAV audio samples and metadata.
#[derive(Debug, Clone)]
pub struct WavData {
    pub metadata: AudioMetadata,
    pub spec: WavSpec,
    pub samples: Vec<i32>,
}

impl WavData {
    /// Reads and parses a PCM WAV file from the filesystem.
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let file_meta = fs::metadata(path_ref)
            .with_context(|| format!("Failed to read metadata for {}", path_ref.display()))?;
        let file_size_bytes = file_meta.len();

        let reader = WavReader::open(path_ref)
            .with_context(|| format!("Failed to open WAV reader for {}", path_ref.display()))?;
        let spec = reader.spec();

        if spec.sample_format != SampleFormat::Int {
            return Err(anyhow!(
                "Unsupported audio format: floating-point WAV is not supported. Use integer PCM WAV."
            ));
        }

        let samples: Vec<i32> = match spec.bits_per_sample {
            8 => {
                let r = WavReader::open(path_ref)?;
                r.into_samples::<i8>()
                    .map(|s| s.map(|v| v as i32))
                    .collect::<Result<Vec<i32>, _>>()?
            }
            16 => {
                let r = WavReader::open(path_ref)?;
                r.into_samples::<i16>()
                    .map(|s| s.map(|v| v as i32))
                    .collect::<Result<Vec<i32>, _>>()?
            }
            24 | 32 => {
                let r = WavReader::open(path_ref)?;
                r.into_samples::<i32>()
                    .collect::<Result<Vec<i32>, _>>()?
            }
            other => {
                return Err(anyhow!(
                    "Unsupported bit depth: {other}-bit. Supported bit depths: 8, 16, 24, 32 bit."
                ));
            }
        };

        let total_samples = samples.len();
        let duration_seconds = if spec.sample_rate > 0 && spec.channels > 0 {
            total_samples as f64 / (spec.sample_rate as f64 * spec.channels as f64)
        } else {
            0.0
        };

        let metadata = AudioMetadata {
            file_path: Some(path_ref.to_path_buf()),
            file_size_bytes,
            sample_rate: spec.sample_rate,
            channels: spec.channels,
            bits_per_sample: spec.bits_per_sample,
            total_samples,
            duration_seconds,
        };

        Ok(Self {
            metadata,
            spec,
            samples,
        })
    }

    /// Writes in-memory samples to a new WAV file.
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path_ref = path.as_ref();
        let mut writer = WavWriter::create(path_ref, self.spec)
            .with_context(|| format!("Failed to create WAV writer at {}", path_ref.display()))?;

        match self.spec.bits_per_sample {
            8 => {
                for &sample in &self.samples {
                    writer.write_sample(sample as i8)?;
                }
            }
            16 => {
                for &sample in &self.samples {
                    writer.write_sample(sample as i16)?;
                }
            }
            24 | 32 => {
                for &sample in &self.samples {
                    writer.write_sample(sample as i32)?;
                }
            }
            other => {
                return Err(anyhow!("Cannot write WAV with unsupported bit depth: {other}"));
            }
        }

        writer.finalize()?;
        Ok(())
    }
}

/// Calculates Signal-to-Noise Ratio (SNR) in dB between original and modified audio samples.
pub fn calculate_snr(original: &[i32], modified: &[i32]) -> f64 {
    if original.len() != modified.len() || original.is_empty() {
        return 0.0;
    }

    let mut signal_power: f64 = 0.0;
    let mut noise_power: f64 = 0.0;

    for (&orig, &alt) in original.iter().zip(modified.iter()) {
        let o = orig as f64;
        let diff = (orig - alt) as f64;
        signal_power += o * o;
        noise_power += diff * diff;
    }

    if noise_power == 0.0 {
        return 999.0; // Infinite SNR (no noise added)
    }

    if signal_power == 0.0 {
        return 0.0;
    }

    10.0 * (signal_power / noise_power).log10()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snr_calculation() {
        let original = vec![1000, 2000, -1000, -2000];
        let identical = vec![1000, 2000, -1000, -2000];
        assert_eq!(calculate_snr(&original, &identical), 999.0);

        let noisy = vec![1001, 1999, -1001, -1999];
        let snr = calculate_snr(&original, &noisy);
        assert!(snr > 60.0);
    }
}
