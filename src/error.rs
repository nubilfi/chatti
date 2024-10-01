use std::error::Error as StdError;
use thiserror::Error;

/// Represents application-wide errors.
#[derive(Debug, Error)]
pub enum Application {
    /// Configuration-related errors.
    #[error("Configuration error: {0}")]
    Config(#[from] crate::config::FSError),

    /// UI-related errors.
    #[error("UI error: {0}")]
    Ui(#[from] color_eyre::Report),

    /// Network-related errors.
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// JSON parsing errors.
    #[error("JSON parsing error: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// Unexpected errors.
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl From<Box<dyn StdError>> for Application {
    fn from(error: Box<dyn StdError>) -> Self {
        Application::Unexpected(error.to_string())
    }
}

impl Application {
    /// Display an error message.
    ///
    /// This method provides a simplified error message suitable for display to the user.
    ///
    /// # Returns
    ///
    /// A string slice containing the user-friendly error message.
    ///
    /// # Examples
    ///
    /// ```
    /// use chatti::error::Application;
    ///
    /// let error = Application::Unexpected("Something went wrong".to_string());
    /// assert_eq!(error.user_friendly_message(), "An unexpected error occurred");
    /// ```
    pub fn display_message(&self) -> &str {
        match self {
            Application::Config(_) => "There was an issue with the application configuration",
            Application::Ui(_) => "An error occurred in the user interface",
            Application::Network(_) => "There was a problem connecting to the server",
            Application::JsonParse(_) => "There was an issue processing the server response",
            Application::Unexpected(_) => "An unexpected error occurred",
        }
    }
}

/// A type alias for Results that use the Application error type.
pub type AppResult<T> = Result<T, Application>;
