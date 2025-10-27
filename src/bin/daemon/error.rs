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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let backend_error = RegmsgError::BackendError { 
            backend: "Test".to_string(), 
            message: "Test error".to_string() 
        };
        assert_eq!(format!("{}", backend_error), "Backend error Test: Test error");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let regmsg_error: RegmsgError = io_error.into();
        match regmsg_error {
            RegmsgError::IoError(_) => (), // OK
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_result_type() {
        fn returns_result() -> Result<String> {
            Ok("success".to_string())
        }
        
        assert!(returns_result().is_ok());
    }
}