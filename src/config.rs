use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

/// Configuration structure for the Chatti application.
///
/// This struct holds the configuration parameters for the application,
/// including API endpoint, model name, streaming flag, and temperature.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// The API endpoint for the chat service.
    pub api_endpoint: String,
    /// The name of the model to use for chat.
    pub model: String,
    /// Whether to use streaming for responses.
    pub stream: bool,
    /// The temperature parameter for text generation.
    pub temperature: f32,
}

impl Config {
    /// Loads the configuration from a file.
    ///
    /// This function attempts to load the configuration from a TOML file
    /// located at `~/.config/chatti/config.toml`. If the file doesn't exist,
    /// it creates a default configuration file.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either the loaded `Config` or a `FSError`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use chatti::config::Config;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = Config::load()?;
    /// println!("API Endpoint: {}", config.api_endpoint);
    /// # Ok(())
    /// # }
    /// ```
    pub fn load() -> Result<Self, FSError> {
        let config_dir = dirs::home_dir()
            .ok_or(FSError::HomeDirNotFound)?
            .join(".config")
            .join("chatti");

        fs::create_dir_all(&config_dir).map_err(FSError::IoError)?;

        let config_path = config_dir.join("config.toml");

        if !config_path.exists() {
            Self::create_default_config(&config_path)?;
        }

        let config_content = fs::read_to_string(&config_path).map_err(FSError::IoError)?;
        let config: Config = toml::from_str(&config_content).map_err(FSError::TomlParseError)?;

        Ok(config)
    }

    /// Creates a default configuration file.
    ///
    /// This function is called when the configuration file doesn't exist.
    /// It creates a new file with default values.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to the `PathBuf` where the config file should be created.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or a `FSError`.
    fn create_default_config(path: &PathBuf) -> Result<(), FSError> {
        let default_config = Config {
            api_endpoint: String::new(),
            model: String::new(),
            stream: false,
            temperature: 0.7,
        };

        let toml_string = toml::to_string(&default_config).map_err(FSError::TomlSerializeError)?;
        fs::write(path, toml_string).map_err(FSError::IoError)?;

        Ok(())
    }
}

/// Represents errors that can occur during configuration operations.
#[derive(Debug, thiserror::Error)]
pub enum FSError {
    /// Error when the home directory cannot be found.
    #[error("home directory not found")]
    HomeDirNotFound,

    /// Error during I/O operations.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error when parsing TOML content.
    #[error("TOML parse error: {0}")]
    TomlParseError(#[from] toml::de::Error),

    /// Error when serializing to TOML.
    #[error("Toml serialize error: {0}")]
    TomlSerializeError(#[from] toml::ser::Error),
}
