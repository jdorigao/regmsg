//! Unified Error Handling System
//!
//! This module defines centralized error types for the entire regmsg application,
//! enabling consistency in error handling across different modules and backends.

use thiserror::Error;

/// Enumeration of all error types in the application
#[derive(Error, Debug)]
pub enum RegmsgError {
    /// Error related to a specific backend (Wayland, DRM, etc.)
    #[error("Backend error {backend}: {message}")]
    BackendError {
        backend: String,
        message: String,
    },
    
    /// Invalid arguments error
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
    
    /// System I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Data conversion error
    #[error("Conversion error: {0}")]
    ConversionError(String),
    
    /// Data parsing error
    #[error("Parse error: {0}")]
    ParseError(String),
    
    /// Resource not found error
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    /// Generic system error
    #[error("System error: {0}")]
    SystemError(String),
}

impl Clone for RegmsgError {
    fn clone(&self) -> Self {
        match self {
            RegmsgError::BackendError { backend, message } => {
                RegmsgError::BackendError {
                    backend: backend.clone(),
                    message: message.clone(),
                }
            }
            RegmsgError::InvalidArguments(msg) => {
                RegmsgError::InvalidArguments(msg.clone())
            }
            RegmsgError::IoError(_) => {
                // For IoError, we'll create a generic version since std::io::Error doesn't implement Clone
                RegmsgError::SystemError("I/O Error".to_string())
            }
            RegmsgError::ConversionError(msg) => {
                RegmsgError::ConversionError(msg.clone())
            }
            RegmsgError::ParseError(msg) => {
                RegmsgError::ParseError(msg.clone())
            }
            RegmsgError::NotFound(msg) => {
                RegmsgError::NotFound(msg.clone())
            }
            RegmsgError::SystemError(msg) => {
                RegmsgError::SystemError(msg.clone())
            }
        }
    }
}

impl From<std::num::ParseIntError> for RegmsgError {
    fn from(error: std::num::ParseIntError) -> Self {
        RegmsgError::ParseError(error.to_string())
    }
}

impl From<std::string::FromUtf8Error> for RegmsgError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        RegmsgError::ConversionError(error.to_string())
    }
}

impl From<chrono::ParseError> for RegmsgError {
    fn from(error: chrono::ParseError) -> Self {
        RegmsgError::ParseError(error.to_string())
    }
}

impl From<toml::ser::Error> for RegmsgError {
    fn from(error: toml::ser::Error) -> Self {
        RegmsgError::SystemError(error.to_string())
    }
}

impl From<toml::de::Error> for RegmsgError {
    fn from(error: toml::de::Error) -> Self {
        RegmsgError::SystemError(error.to_string())
    }
}

/// Standardized result type for the entire application
pub type Result<T> = std::result::Result<T, RegmsgError>;