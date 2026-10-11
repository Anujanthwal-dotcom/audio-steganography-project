use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct AudioMetadata {
    pub file_path: Option<PathBuf>,
    pub file_size_bytes: u64,
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
    pub total_samples: usize,
    pub duration_seconds: f64,
}

impl AudioMetadata {
    pub fn duration_formatted(&self) -> String {
        let total_secs = self.duration_seconds as u64;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        let millis = ((self.duration_seconds - total_secs as f64) * 100.0) as u64;
        format!("{:02}:{:02}.{:02}", mins, secs, millis)
    }

    pub fn max_capacity_bytes(&self) -> usize {
        self.total_samples / 8
    }

    pub fn formatted_size(&self) -> String {
        let bytes = self.file_size_bytes;
        if bytes >= 1024 * 1024 {
            format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
        } else if bytes >= 1024 {
            format!("{:.2} KB", bytes as f64 / 1024.0)
        } else {
            format!("{} B", bytes)
        }
    }
}
