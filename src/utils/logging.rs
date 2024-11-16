use tracing::{Level, Subscriber};
use tracing_subscriber::{FmtSubscriber, EnvFilter};
use crate::utils::error::{ElevateError, ElevateResult};
use crate::utils::config::{Config, LogFormat};

pub fn init() -> ElevateResult<()> {
    let config = Config::load()?;
    let level = get_log_level(&config.logging.level)?;
    
    FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive(level.into()))
        .with_target(false)
        .with_file(config.logging.file.is_some())
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_writer(move || get_writer(&config).unwrap())
        .with_ansi(true)
       // .pretty(matches!(config.logging.format, LogFormat::Pretty))
        .try_init()
        .map_err(|e| ElevateError::ConfigError(format!("Failed to initialize logger: {}", e)))?;

    Ok(())
}

fn get_log_level(level: &str) -> ElevateResult<Level> {
    match level.to_lowercase().as_str() {
        "trace" => Ok(Level::TRACE),
        "debug" => Ok(Level::DEBUG),
        "info" => Ok(Level::INFO),
        "warn" => Ok(Level::WARN),
        "error" => Ok(Level::ERROR),
        _ => Err(ElevateError::ConfigError(format!("Invalid log level: {}", level)))
    }
}

fn get_writer(config: &Config) -> ElevateResult<Box<dyn std::io::Write + Send + Sync>> {
    if let Some(path) = &config.logging.file {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| ElevateError::ConfigError(format!("Failed to open log file: {}", e)))?;
        Ok(Box::new(file))
    } else {
        Ok(Box::new(std::io::stdout()))
    }
}