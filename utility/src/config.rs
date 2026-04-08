use serde::Deserialize;
use serde::de::{self, Deserializer};
use std::{fmt, fs, path::Path};

pub const DEFAULT_CONFIG_PATH: &str = "config.yml";

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub min_viewport_width: u32,
    #[serde(deserialize_with = "deserialize_aspect_ratio")]
    pub aspect_ratio: f64,
    pub preview_width: u32,
    pub preview_samples_per_pixel: u32,
    pub preview_max_bounces: u32,
    pub width: u32,
    pub threads: u8,
    pub frame_time: u64,
    pub samples_per_pixel: u32,
    pub max_bounces: u32,
    pub camera_move_step: f64,
    pub camera_look_step_radians: f64,
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
        if self.min_viewport_width == 0 {
            return Err(ConfigError::Validation(
                "`min_viewport_width` must be greater than 0".into(),
            ));
        }
        if self.aspect_ratio <= 0.0 {
            return Err(ConfigError::Validation(
                "`aspect_ratio` must be greater than 0".into(),
            ));
        }
        if self.preview_width == 0 {
            return Err(ConfigError::Validation(
                "`preview_width` must be greater than 0".into(),
            ));
        }
        if self.width == 0 {
            return Err(ConfigError::Validation(
                "`width` must be greater than 0".into(),
            ));
        }
        if self.preview_samples_per_pixel == 0 {
            return Err(ConfigError::Validation(
                "`preview_samples_per_pixel` must be greater than 0".into(),
            ));
        }
        if self.preview_max_bounces == 0 {
            return Err(ConfigError::Validation(
                "`preview_max_bounces` must be greater than 0".into(),
            ));
        }
        if self.samples_per_pixel == 0 {
            return Err(ConfigError::Validation(
                "`samples_per_pixel` must be greater than 0".into(),
            ));
        }
        if self.max_bounces == 0 {
            return Err(ConfigError::Validation(
                "`max_bounces` must be greater than 0".into(),
            ));
        }
        if !self.camera_move_step.is_finite() || self.camera_move_step <= 0.0 {
            return Err(ConfigError::Validation(
                "`camera_move_step` must be a finite value greater than 0".into(),
            ));
        }
        if !self.camera_look_step_radians.is_finite() || self.camera_look_step_radians <= 0.0 {
            return Err(ConfigError::Validation(
                "`camera_look_step_radians` must be a finite value greater than 0".into(),
            ));
        }
        if self.threads == 0 {
            return Err(ConfigError::Validation(
                "`threads` must be greater than 0".into(),
            ));
        }
        if self.threads > 32 {
            return Err(ConfigError::Validation(
                "you cannot use more threads than 32".into(),
            ));
        }
        if self.height_from_width(self.min_viewport_width) == 0 {
            return Err(ConfigError::Validation(
                "calculated viewport height must be greater than 0".into(),
            ));
        }

        Ok(())
    }

    pub fn height_from_width(&self, width: u32) -> u32 {
        ((width as f64) / self.aspect_ratio).round().max(1.0) as u32
    }

    pub fn min_viewport_height(&self) -> u32 {
        self.height_from_width(self.min_viewport_width)
    }

    pub fn preview_height(&self) -> u32 {
        self.height_from_width(self.preview_width)
    }

    pub fn final_height(&self) -> u32 {
        self.height_from_width(self.width)
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

#[derive(Deserialize)]
#[serde(untagged)]
enum AspectRatioValue {
    Number(f64),
    Text(String),
}

fn deserialize_aspect_ratio<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = AspectRatioValue::deserialize(deserializer)?;
    match value {
        AspectRatioValue::Number(v) if v > 0.0 => Ok(v),
        AspectRatioValue::Number(_) => Err(de::Error::custom("aspect_ratio must be > 0")),
        AspectRatioValue::Text(text) => parse_aspect_ratio(&text).map_err(de::Error::custom),
    }
}

fn parse_aspect_ratio(value: &str) -> Result<f64, String> {
    let trimmed = value.trim();
    if let Some((num, den)) = trimmed.split_once('/') {
        let numerator = num
            .trim()
            .parse::<f64>()
            .map_err(|_| format!("invalid aspect ratio numerator `{num}`"))?;
        let denominator = den
            .trim()
            .parse::<f64>()
            .map_err(|_| format!("invalid aspect ratio denominator `{den}`"))?;

        if numerator <= 0.0 || denominator <= 0.0 {
            return Err("aspect ratio values must be > 0".into());
        }

        return Ok(numerator / denominator);
    }

    let ratio = trimmed
        .parse::<f64>()
        .map_err(|_| format!("invalid aspect ratio `{value}`"))?;
    if ratio <= 0.0 {
        return Err("aspect ratio must be > 0".into());
    }

    Ok(ratio)
}
