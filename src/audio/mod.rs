pub mod metadata;
pub mod wav;

pub use metadata::AudioMetadata;
pub use wav::{calculate_snr, WavData};

/// Formats an integer with comma separators for clean display (e.g. 1,323,000)
pub fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for (i, &c) in chars.iter().enumerate() {
        result.push(c);
        let remaining = len - i - 1;
        if remaining > 0 && remaining % 3 == 0 {
            result.push(',');
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(42), "42");
        assert_eq!(format_number(1234), "1,234");
        assert_eq!(format_number(1323000), "1,323,000");
    }
}
