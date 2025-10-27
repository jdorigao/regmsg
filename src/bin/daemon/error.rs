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
    use std::io;

    /// Test creating different types of RegmsgError
    #[test]
    fn test_error_creation() {
        let backend_error = RegmsgError::BackendError { 
            backend: "Test".to_string(), 
            message: "Test error".to_string() 
        };
        assert_eq!(format!("{}", backend_error), "Backend error Test: Test error");
    }

    /// Test conversion from std::io::Error to RegmsgError
    #[test]
    fn test_io_error_conversion() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let regmsg_error: RegmsgError = io_error.into();
        match regmsg_error {
            RegmsgError::IoError(_) => (), // Expected
            _ => panic!("Expected IoError"),
        }
    }

    /// Test the Result type alias
    #[test]
    fn test_result_type() {
        fn returns_result() -> Result<String> {
            Ok("success".to_string())
        }
        
        assert!(returns_result().is_ok());
        assert_eq!(returns_result().unwrap(), "success");
    }

    /// Test all error variants creation
    #[test]
    fn test_all_error_variants() {
        let backend_error = RegmsgError::BackendError {
            backend: "TestBackend".to_string(),
            message: "Test message".to_string(),
        };
        assert!(format!("{}", backend_error).contains("TestBackend"));
        assert!(format!("{}", backend_error).contains("Test message"));

        let invalid_args_error = RegmsgError::InvalidArguments("Invalid args".to_string());
        assert!(format!("{}", invalid_args_error).contains("Invalid arguments:"));

        let conversion_error = RegmsgError::ConversionError("Conversion failed".to_string());
        assert!(format!("{}", conversion_error).contains("Conversion error:"));

        let parse_error = RegmsgError::ParseError("Parse failed".to_string());
        assert!(format!("{}", parse_error).contains("Parse error:"));

        let not_found_error = RegmsgError::NotFound("Resource".to_string());
        assert!(format!("{}", not_found_error).contains("Resource not found:"));

        let system_error = RegmsgError::SystemError("System error".to_string());
        assert!(format!("{}", system_error).contains("System error:"));
    }

    /// Test conversion from ParseIntError
    #[test]
    fn test_parse_int_error_conversion() {
        let parse_error = "not_a_number".parse::<i32>().unwrap_err();
        let regmsg_error: RegmsgError = parse_error.into();
        match regmsg_error {
            RegmsgError::ParseError(_) => (), // Expected
            _ => panic!("Expected ParseError from integer parsing"),
        }
    }

    /// Test conversion from FromUtf8Error
    #[test]
    fn test_from_utf8_error_conversion() {
        let invalid_utf8 = vec![0, 159, 146, 150]; // Invalid UTF-8 sequence
        let utf8_error = String::from_utf8(invalid_utf8).unwrap_err();
        let regmsg_error: RegmsgError = utf8_error.into();
        match regmsg_error {
            RegmsgError::ConversionError(_) => (), // Expected
            _ => panic!("Expected ConversionError from UTF-8 conversion"),
        }
    }

    /// Test conversion from chrono::ParseError
    #[test]
    fn test_chrono_error_conversion() {
        let result = "invalid".parse::<chrono::DateTime<chrono::Utc>>();
        if let Err(chrono_error) = result {
            let regmsg_error: RegmsgError = chrono_error.into();
            match regmsg_error {
                RegmsgError::ParseError(_) => (), // Expected
                _ => panic!("Expected ParseError from chrono conversion"),
            }
        }
    }

    /// Test conversion from TOML serialization error
    #[test]
    fn test_toml_ser_error_conversion() {
        let result = toml::to_string(&std::f64::NAN);  // NaN cannot be serialized to TOML
        if let Err(toml_error) = result {
            let regmsg_error: RegmsgError = toml_error.into();
            match regmsg_error {
                RegmsgError::SystemError(_) => (), // Expected
                _ => panic!("Expected SystemError from TOML serialization"),
            }
        }
    }

    /// Test conversion from TOML deserialization error
    #[test]
    fn test_toml_de_error_conversion() {
        let invalid_toml = "invalid toml [";
        let result: std::result::Result<toml::Value, toml::de::Error> = toml::from_str(invalid_toml);
        if let Err(toml_error) = result {
            let regmsg_error: RegmsgError = toml_error.into();
            match regmsg_error {
                RegmsgError::SystemError(_) => (), // Expected
                _ => panic!("Expected SystemError from TOML deserialization"),
            }
        }
    }

    /// Test error propagation behavior
    #[test]
    fn test_error_propagation() {
        fn operation_that_fails() -> Result<String> {
            Err(RegmsgError::NotFound("Item not found".to_string()))
        }

        let result = operation_that_fails();
        assert!(result.is_err());
        
        match result {
            Err(RegmsgError::NotFound(msg)) => {
                assert_eq!(msg, "Item not found");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    /// Test nested error handling
    #[test]
    fn test_nested_error_handling() {
        fn parse_number(input: &str) -> Result<i32> {
            match input.parse::<i32>() {
                Ok(num) => Ok(num),
                Err(e) => Err(RegmsgError::from(e)),
            }
        }

        let result = parse_number("not_a_number");
        assert!(result.is_err());
        
        match result {
            Err(RegmsgError::ParseError(_)) => (), // Expected
            _ => panic!("Expected ParseError"),
        }
    }

    /// Test error message formatting
    #[test]
    fn test_error_formatting() {
        let error_cases = vec![
            (RegmsgError::BackendError { backend: "KMS".to_string(), message: "Failed".to_string() }, "Backend error KMS: Failed"),
            (RegmsgError::InvalidArguments("Bad input".to_string()), "Invalid arguments: Bad input"),
            (RegmsgError::ConversionError("Failed".to_string()), "Conversion error: Failed"),
            (RegmsgError::ParseError("Syntax error".to_string()), "Parse error: Syntax error"),
            (RegmsgError::NotFound("Resource".to_string()), "Resource not found: Resource"),
            (RegmsgError::SystemError("General error".to_string()), "System error: General error"),
        ];

        for (error, expected) in error_cases {
            assert_eq!(format!("{}", error), expected);
        }
    }

    /// Test IO error with different error kinds
    #[test]
    fn test_io_error_kinds() {
        let kinds = [
            (io::ErrorKind::NotFound, "not found"),
            (io::ErrorKind::PermissionDenied, "denied"),
            (io::ErrorKind::ConnectionRefused, "refused"),
            (io::ErrorKind::AlreadyExists, "exists"),
        ];

        for (kind, _expected_in_msg) in kinds.iter() {
            let io_error = io::Error::new(*kind, "test message");
            let regmsg_error: RegmsgError = io_error.into();
            
            match regmsg_error {
                RegmsgError::IoError(_) => (), // Expected for all IO errors
                _ => panic!("Expected IoError for kind {:?}", kind),
            }
        }
    }
}