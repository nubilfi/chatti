use std::path::PathBuf;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*, EnvFilter};

/// Sets up logging for the application.
///
/// This function initializes a daily rolling file appender for logging,
/// configures the logging format, and sets up the tracing subscriber.
///
/// # Returns
///
/// Returns a `Result` containing the `WorkerGuard` for the non-blocking writer,
/// or a boxed error if setup fails.
///
/// # Examples
///
/// ```
/// use chatti::logging;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let _guard = logging::setup_logging()?;
/// // The guard should be held until the end of the program
/// # Ok(())
/// # }
/// ```
pub fn setup_logging(
) -> Result<tracing_appender::non_blocking::WorkerGuard, Box<dyn std::error::Error>> {
    let config_dir = dirs::home_dir()
        .ok_or("home directory not found")?
        .join(".config")
        .join("chatti");
    std::fs::create_dir_all(&config_dir)?;

    let log_dir = config_dir.join("logs");
    std::fs::create_dir_all(&log_dir)?;

    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "chatti.log");

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_span_events(FmtSpan::FULL),
        )
        .init();
    Ok(guard)
}

/// Returns the path to the log file.
///
/// # Returns
///
/// A `PathBuf` representing the path to the log file.
///
/// # Examples
///
/// ```
/// use chatti::logging;
///
/// let log_path = logging::get_log_file_path();
/// println!("Log file is located at: {:?}", log_path);
/// ```
pub fn get_log_file_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".config")
        .join("chatti")
        .join("logs")
        .join("errors.log")
}
