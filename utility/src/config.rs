
use std::{error::Error, fmt, fs, path::Path};
use serde::Deserialize;

pub const DEFAULT_CONFIG_PATH: &str = "config.yml";

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub width: u32,
    pub height: u32,
    pub threads: u8,
    pub frame_time: u64,
    pub samples_per_pixel: u32,
    pub max_bounces: u32
}

#[derive(Debug)]
pub enum ConfigError {
    Read {
        path: String,
        source: std::io::Error,
    },
    Parse {
        path: String,
        source: serde_yaml::Error,
    },
    Validation(String),
}

impl AppConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path_ref = path.as_ref();
        let path_display = path_ref.display().to_string();

        let raw = fs::read_to_string(path_ref).map_err(|source| ConfigError::Read {
            path: path_display.clone(),
            source,
        })?;

        let config: Self = serde_yaml::from_str(&raw).map_err(|source| ConfigError::Parse {
            path: path_display,
            source,
        })?;

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.width == 0 {
            return Err(ConfigError::Validation("`width` must be greater than 0".into()));
        }
        if self.height == 0 {
            return Err(ConfigError::Validation("`height` must be greater than 0".into()));
        }
        if self.threads > 32 {
            return Err(ConfigError::Validation("you cannot use more threads than 32".into()));
        }

        Ok(())
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(f, "failed to read config file `{path}`: {source}")
            }
            Self::Parse { path, source } => {
                write!(f, "failed to parse yaml config `{path}`: {source}")
            }
            Self::Validation(message) => write!(f, "invalid config value: {message}"),
        }
    }
}

