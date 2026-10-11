/// Application configuration and preferences.
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub default_output_suffix: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            default_output_suffix: "_stego.wav".to_string(),
        }
    }
}
