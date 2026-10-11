pub fn format_bytes(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} Bytes", bytes)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CapacityStats {
    pub total_samples: usize,
    pub max_capacity_bytes: usize,
    pub payload_bytes: usize,
    pub overhead_bytes: usize,
    pub usage_percentage: f64,
}

impl CapacityStats {
    pub fn calculate(total_samples: usize, payload_bytes: usize, overhead_bytes: usize) -> Self {
        let max_capacity_bytes = total_samples / 8;
        let total_required = payload_bytes + overhead_bytes;
        let usage_percentage = if max_capacity_bytes == 0 {
            0.0
        } else {
            (total_required as f64 / max_capacity_bytes as f64) * 100.0
        };

        Self {
            total_samples,
            max_capacity_bytes,
            payload_bytes,
            overhead_bytes,
            usage_percentage,
        }
    }

    pub fn fits(&self) -> bool {
        self.payload_bytes + self.overhead_bytes <= self.max_capacity_bytes
    }

    pub fn status_text(&self) -> &'static str {
        if !self.fits() {
            "Capacity Exceeded"
        } else if self.usage_percentage > 80.0 {
            "Warning: High Density"
        } else {
            "Safe"
        }
    }

    pub fn formatted_max_capacity(&self) -> String {
        format_bytes(self.max_capacity_bytes)
    }

    pub fn formatted_payload_size(&self) -> String {
        format_bytes(self.payload_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capacity_calculation() {
        let stats = CapacityStats::calculate(8000, 100, 58);
        assert_eq!(stats.max_capacity_bytes, 1000);
        assert!(stats.fits());
        assert_eq!(stats.status_text(), "Safe");
        assert!((stats.usage_percentage - 15.8).abs() < 0.1);
    }
}
