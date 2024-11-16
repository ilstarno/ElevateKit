use std::fs;
use std::path::PathBuf;
use crate::utils::error::{ElevateError, ElevateResult};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Text,
    Pretty,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IsolationLevel {
    None,
    Low,
    Medium,
    High,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IntegrityLevel {
    Untrusted,
    Low,
    Medium,
    High,
    System,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
    pub process: ProcessConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<PathBuf>,
    pub format: LogFormat,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub allowed_privileges: Vec<String>,
    pub isolation_level: IsolationLevel,
    pub integrity_level: IntegrityLevel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessConfig {
    pub default_shell: String,
    pub creation_flags: u32,
    pub timeout: Option<u64>,
}

impl Config {
    pub fn load() -> ElevateResult<Self> {
        let config_path = Self::get_config_path()?;
        
        if !config_path.exists() {
            return Ok(Self::default());
        }

        fs::read_to_string(&config_path)
            .map_err(|e| ElevateError::ConfigError(format!("Failed to read config: {}", e)))
            .and_then(|contents| {
                ::serde_json::from_str(&contents)
                    .map_err(|e| ElevateError::ConfigError(format!("Failed to parse config: {}", e)))
            })
    }

    fn get_config_path() -> ElevateResult<PathBuf> {
        let exe_path = std::env::current_exe()
            .map_err(|e| ElevateError::ConfigError(format!("Failed to get executable path: {}", e)))?;
        
        let mut config_path = exe_path.parent()
            .ok_or_else(|| ElevateError::ConfigError("Failed to get parent directory".into()))?
            .to_path_buf();
            
        config_path.push("config.json");
        Ok(config_path)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            logging: LoggingConfig {
                level: "info".to_string(),
                file: None,
                format: LogFormat::Text,
            },
            security: SecurityConfig {
                allowed_privileges: vec![],
                isolation_level: IsolationLevel::Medium,
                integrity_level: IntegrityLevel::Medium,
            },
            process: ProcessConfig {
                default_shell: "powershell.exe".to_string(),
                creation_flags: 0,
                timeout: Some(30),
            },
        }
    }
}