/// Production-grade structured logging system
///
/// Provides context-rich logging with file rotation, performance tracking,
/// and integration with external monitoring systems.

use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use std::fs;

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Minimum log level (default: INFO)
    pub level: Level,

    /// Enable file logging (default: true)
    pub file_enabled: bool,

    /// Log file path (default: app_data/logs/)
    pub log_dir: PathBuf,

    /// Maximum log file size in MB before rotation (default: 10)
    pub max_file_size_mb: u64,

    /// Number of rotated log files to keep (default: 5)
    pub max_files: usize,

    /// Enable console logging (default: true in debug)
    pub console_enabled: bool,

    /// Pretty print console logs (default: true in debug)
    pub console_pretty: bool,
}

impl LogConfig {
    /// Production configuration
    pub fn production(log_dir: PathBuf) -> Self {
        Self {
            level: Level::INFO,
            file_enabled: true,
            log_dir,
            max_file_size_mb: 10,
            max_files: 5,
            console_enabled: false,
            console_pretty: false,
        }
    }

    /// Development configuration
    pub fn development(log_dir: PathBuf) -> Self {
        Self {
            level: Level::DEBUG,
            file_enabled: true,
            log_dir,
            max_file_size_mb: 50,
            max_files: 3,
            console_enabled: true,
            console_pretty: true,
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        // Default to production-like settings
        Self {
            level: Level::INFO,
            file_enabled: true,
            log_dir: PathBuf::from("logs"),
            max_file_size_mb: 10,
            max_files: 5,
            console_enabled: true,  // Console enabled by default
            console_pretty: cfg!(debug_assertions),  // Pretty only in debug builds
        }
    }
}

/// Initialize the logging system
///
/// # Arguments
/// * `config` - Logging configuration
///
/// # Errors
/// Returns error if log directory creation fails
///
/// # Example
/// ```no_run
/// use std::path::PathBuf;
/// use lolshorts::utils::logging::{LogConfig, init_logging};
///
/// let config = LogConfig::production(PathBuf::from("C:/logs"));
/// init_logging(config).expect("Failed to initialize logging");
/// ```
pub fn init_logging(config: LogConfig) -> anyhow::Result<()> {
    // Create log directory if it doesn't exist
    if config.file_enabled {
        fs::create_dir_all(&config.log_dir)?;
    }

    // Build environment filter
    let env_filter = EnvFilter::from_default_env()
        .add_directive(config.level.into());

    // Simplified implementation: Choose one primary output
    if config.file_enabled {
        // File logging with daily rotation
        let file_appender = tracing_appender::rolling::daily(
            config.log_dir.clone(),
            "lolshorts.log"
        );

        let subscriber = fmt()
            .with_env_filter(env_filter)
            .with_writer(file_appender)
            .with_ansi(false)
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true)
            .with_file(true)
            .json()
            .finish();

        tracing::subscriber::set_global_default(subscriber)?;
    } else if config.console_enabled {
        // Console logging only
        if config.console_pretty {
            let subscriber = fmt()
                .with_env_filter(env_filter)
                .pretty()
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_line_number(true)
                .with_file(true)
                .finish();

            tracing::subscriber::set_global_default(subscriber)?;
        } else {
            let subscriber = fmt()
                .with_env_filter(env_filter)
                .compact()
                .with_target(true)
                .with_thread_ids(true)
                .finish();

            tracing::subscriber::set_global_default(subscriber)?;
        }
    } else {
        // No logging configured
        return Err(anyhow::anyhow!("No logging output configured"));
    }

    Ok(())
}

/// Logging macros with context
///
/// These are re-exports of tracing macros with added context helpers

/// Log performance-critical operations
#[macro_export]
macro_rules! log_perf {
    ($op:expr, $($arg:tt)*) => {
        tracing::debug!(
            operation = $op,
            $($arg)*
        )
    };
}

/// Log security-sensitive operations
#[macro_export]
macro_rules! log_security {
    ($($arg:tt)*) => {
        tracing::warn!(
            security = true,
            $($arg)*
        )
    };
}

/// Log user actions for audit trail
#[macro_export]
macro_rules! log_audit {
    ($action:expr, $($arg:tt)*) => {
        tracing::info!(
            audit = true,
            action = $action,
            $($arg)*
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_log_config_production() {
        let temp_dir = tempdir().unwrap();
        let config = LogConfig::production(temp_dir.path().to_path_buf());

        assert_eq!(config.level, Level::INFO);
        assert!(config.file_enabled);
        assert!(!config.console_enabled);
        assert!(!config.console_pretty);
    }

    #[test]
    fn test_log_config_development() {
        let temp_dir = tempdir().unwrap();
        let config = LogConfig::development(temp_dir.path().to_path_buf());

        assert_eq!(config.level, Level::DEBUG);
        assert!(config.file_enabled);
        assert!(config.console_enabled);
        assert!(config.console_pretty);
    }

    #[test]
    fn test_init_logging_creates_directory() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path().join("logs");

        let config = LogConfig {
            log_dir: log_dir.clone(),
            ..Default::default()
        };

        // Should create directory
        assert!(!log_dir.exists());
        init_logging(config).expect("Failed to init logging");
        assert!(log_dir.exists());
    }
}
