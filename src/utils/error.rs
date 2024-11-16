use thiserror::Error;
use std::fmt;

#[derive(Debug)]
pub enum TokenErrorKind {
    OpenProcessToken,
    AdjustPrivileges,
    LookupPrivilege,
    Open,
    ProcessOperation,
}

#[derive(Debug)]
pub enum PrivilegeErrorKind {
    Enable,
    Disable,
    Query,
}

impl fmt::Display for TokenErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProcessOperation => write!(f, "Process operation failed"),
            Self::OpenProcessToken => write!(f, "Failed to open process token"),
            Self::AdjustPrivileges => write!(f, "Failed to adjust token privileges"),
            Self::LookupPrivilege => write!(f, "Failed to lookup privilege"),
            Self::Open => write!(f, "Failed to open token"),
        }
    }
}

impl fmt::Display for PrivilegeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Enable => write!(f, "Failed to enable privilege"),
            Self::Disable => write!(f, "Failed to disable privilege"),
            Self::Query => write!(f, "Failed to query privilege"),
        }
    }
}

#[derive(Debug)]
pub enum WindowsErrorKind {
    ProcessOperation,
    TokenOperation,
    SecurityOperation,
    ServiceOperation,
    PrivilegeOperation,
    IntegrityOperation,
    RegistryOperation,
    FileOperation,
    MemoryOperation,
    DebugOperation,
}

impl fmt::Display for WindowsErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProcessOperation => write!(f, "Process operation failed"),
            Self::TokenOperation => write!(f, "Token operation failed"),
            Self::SecurityOperation => write!(f, "Security operation failed"),
            Self::ServiceOperation => write!(f, "Service operation failed"),
            Self::PrivilegeOperation => write!(f, "Privilege operation failed"),
            Self::IntegrityOperation => write!(f, "Integrity level operation failed"),
            Self::RegistryOperation => write!(f, "Registry operation failed"),
            Self::FileOperation => write!(f, "File operation failed"),
            Self::MemoryOperation => write!(f, "Memory operation failed"),
            Self::DebugOperation => write!(f, "Debug operation failed"),
        }
    }
}

#[derive(Debug)]
pub struct WindowsError {
    code: i32,
}

impl WindowsError {
    pub fn last_error() -> Self {
        use winapi::um::errhandlingapi::GetLastError;
        Self {
            code: unsafe { GetLastError() as i32 }
        }
    }

    pub fn to_elevate_error(&self, kind: WindowsErrorKind, context: &str) -> ElevateError {
        // Implement error conversion logic here
        // For now, just create a basic error wrapper
        ElevateError::Windows {
            kind,
            code: self.code,
            context: context.to_string(),
        }
    }
}

#[derive(Error, Debug)]
pub enum ElevateError {
    #[error("Token error: {kind}")]
    TokenError {
        kind: TokenErrorKind,
        #[source]
        source: Option<Box<dyn std::error::Error>>,
    },

    #[error("Privilege error: {kind}")]
    PrivilegeError {
        kind: PrivilegeErrorKind,
        privilege: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error>>,
    },

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(std::io::Error),

    #[error("Windows error: {kind} (code: {code})")]
    Windows {
        kind: WindowsErrorKind,
        code: i32,
        context: String,
    },

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Process error: {0}")]
    ProcessError(String),
}

impl From<std::io::Error> for ElevateError {
    fn from(error: std::io::Error) -> Self {
        ElevateError::IoError(error)
    }
}

pub type ElevateResult<T> = Result<T, ElevateError>;
