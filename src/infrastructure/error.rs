use thiserror::Error;

#[derive(Error, Debug)]
pub enum TiError {
    #[error("Service error: {0}")]
    ServiceError(String),
    
    #[error("Process error: {0}")]
    ProcessError(String),
    
    #[error("Privilege error: {0}")]
    PrivilegeError(String),
    
    #[error("System error: {0}")]
    SystemError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type TiResult<T> = Result<T, TiError>; 