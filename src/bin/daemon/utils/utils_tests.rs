// Comprehensive test suite for the utils module
// This file contains tests for all utility components in the regmsg daemon.

use crate::utils::error::{RegmsgError, Result};

// Tests for the error module functionality
#[cfg(test)]
mod error_tests {
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

    /// Test error source chaining
    #[test]
    fn test_error_source_chaining() {
        use std::error::Error;
        
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let regmsg_error: RegmsgError = io_error.into();
        
        match regmsg_error {
            RegmsgError::IoError(source) => {
                assert!(source.source().is_none()); // io::Error doesn't have a source in this case
            }
            _ => panic!("Expected IoError"),
        }
    }

    /// Test custom error creation with complex messages
    #[test]
    fn test_custom_error_creation() {
        let complex_message = "This is a complex error message with \"quotes\" and \n newlines";
        let backend_error = RegmsgError::BackendError {
            backend: "ComplexBackend".to_string(),
            message: complex_message.to_string(),
        };
        
        let formatted = format!("{}", backend_error);
        assert!(formatted.contains("ComplexBackend"));
        assert!(formatted.contains(complex_message));
    }



    /// Test error debugging output
    #[test]
    fn test_error_debug_output() {
        let error = RegmsgError::NotFound("test_resource".to_string());
        let debug_output = format!("{:?}", error);
        
        // Should contain the error variant name
        assert!(debug_output.contains("NotFound"));
        // Should contain the resource name
        assert!(debug_output.contains("test_resource"));
    }

    /// Test error serialization compatibility
    #[test]
    fn test_error_serialization_compatibility() {
        // Test that errors can be sent between threads
        let error = RegmsgError::SystemError("thread test".to_string());
        
        let handle = std::thread::spawn(move || {
            format!("{}", error)
        });
        
        let result = handle.join().unwrap();
        assert_eq!(result, "System error: thread test");
    }

    /// Test error composition with multiple layers
    #[test]
    fn test_error_composition() {
        fn layer1() -> Result<String> {
            Err(RegmsgError::ParseError("layer1 error".to_string()))
        }
        
        fn layer2() -> Result<String> {
            match layer1() {
                Ok(value) => Ok(value),
                Err(RegmsgError::ParseError(msg)) => {
                    Err(RegmsgError::SystemError(format!("layer2 wrapping: {}", msg)))
                }
                Err(e) => Err(e),
            }
        }
        
        let result = layer2();
        assert!(result.is_err());
        
        match result {
            Err(RegmsgError::SystemError(msg)) => {
                assert!(msg.contains("layer2 wrapping:"));
                assert!(msg.contains("layer1 error"));
            }
            _ => panic!("Expected SystemError wrapping ParseError"),
        }
    }

    /// Test all From implementations
    #[test]
    fn test_all_from_implementations() {
        // Test From<std::io::Error>
        let io_error: RegmsgError = std::io::Error::new(std::io::ErrorKind::Other, "test").into();
        assert!(matches!(io_error, RegmsgError::IoError(_)));

        // Test From<std::num::ParseIntError>
        let parse_error: RegmsgError = "not_a_number".parse::<i32>().unwrap_err().into();
        assert!(matches!(parse_error, RegmsgError::ParseError(_)));

        // Test From<std::string::FromUtf8Error>
        let utf8_error: RegmsgError = String::from_utf8(vec![0, 159, 146, 150]).unwrap_err().into();
        assert!(matches!(utf8_error, RegmsgError::ConversionError(_)));

        // Test From<chrono::ParseError>
        let chrono_error: RegmsgError = "invalid".parse::<chrono::DateTime<chrono::Utc>>().unwrap_err().into();
        assert!(matches!(chrono_error, RegmsgError::ParseError(_)));

        // Test From<toml::ser::Error>
        let toml_ser_error: RegmsgError = toml::to_string(&std::f64::NAN).unwrap_err().into();
        assert!(matches!(toml_ser_error, RegmsgError::SystemError(_)));

        // Test From<toml::de::Error>
        let toml_de_error: RegmsgError = toml::from_str::<toml::Value>("invalid [").unwrap_err().into();
        assert!(matches!(toml_de_error, RegmsgError::SystemError(_)));
    }

    /// Test edge cases in error creation
    #[test]
    fn test_edge_cases() {
        // Empty strings
        let empty_error = RegmsgError::InvalidArguments("".to_string());
        assert_eq!(format!("{}", empty_error), "Invalid arguments: ");

        // Very long strings
        let long_message = "A".repeat(10000);
        let long_error = RegmsgError::SystemError(long_message.clone());
        assert!(format!("{}", long_error).contains(&long_message));

        // Special characters
        let special_chars = r#"Special chars: \n\t\r\"'"#;
        let special_error = RegmsgError::BackendError {
            backend: "Test".to_string(),
            message: special_chars.to_string(),
        };
        assert!(format!("{}", special_error).contains(special_chars));
    }

    /// Test error conversion performance (basic sanity check)
    #[test]
    fn test_error_conversion_performance() {
        use std::time::Instant;
        
        let start = Instant::now();
        for _ in 0..1000 {
            let io_error = io::Error::new(io::ErrorKind::NotFound, "test");
            let _regmsg_error: RegmsgError = io_error.into();
        }
        let duration = start.elapsed();
        
        // This is just a basic sanity check - should complete in reasonable time
        assert!(duration.as_millis() < 1000, "Error conversion took too long");
    }
}

// Integration tests for utils module components
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    /// Test that utils module components work together
    #[test]
    fn test_module_integration() {
        // Test that we can create and use errors from the error module
        let error = RegmsgError::BackendError {
            backend: "IntegrationTest".to_string(),
            message: "Testing module integration".to_string(),
        };
        
        let result: Result<()> = Err(error);
        assert!(result.is_err());
        
        match result {
            Err(RegmsgError::BackendError { backend, message }) => {
                assert_eq!(backend, "IntegrationTest");
                assert_eq!(message, "Testing module integration");
            }
            _ => panic!("Expected BackendError"),
        }
    }
    
    /// Test that Result type alias works correctly
    #[test]
    fn test_result_type_alias() {
        fn process_data(success: bool) -> Result<String> {
            if success {
                Ok("Processed successfully".to_string())
            } else {
                Err(RegmsgError::SystemError("Processing failed".to_string()))
            }
        }
        
        let success_result = process_data(true);
        assert!(success_result.is_ok());
        assert_eq!(success_result.unwrap(), "Processed successfully");
        
        let failure_result = process_data(false);
        assert!(failure_result.is_err());
        
        match failure_result {
            Err(RegmsgError::SystemError(msg)) => {
                assert_eq!(msg, "Processing failed");
            }
            _ => panic!("Expected SystemError"),
        }
    }
}

// Tests for the tracing module functionality
#[cfg(test)]
mod tracing_tests {
    use std::sync::Once;
    use tempfile::NamedTempFile;
    use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

    static TEST_INIT: Once = Once::new();

    #[test]
    fn test_setup_tracing() {
        TEST_INIT.call_once(|| {
            // Create a temporary file for testing
            let temp_file = NamedTempFile::new().expect("Failed to create temp file");
            let file = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(temp_file.path())
                .expect("Failed to open temp log file");

            // Create non-blocking writer for testing
            let (non_blocking, _guard) = tracing_appender::non_blocking::NonBlocking::new(file);

            let env_filter = EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")); // Default to info level if not set

            // Configure file layer for testing
            let file_layer = fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_filter(env_filter.clone());

            // Configure stdout layer for testing
            let stdout_layer = fmt::layer()
                .with_writer(std::io::stdout)
                .with_ansi(true)
                .with_filter(env_filter);

            // Set up the subscriber with both file and console output for testing
            tracing_subscriber::registry()
                .with(file_layer)
                .with(stdout_layer)
                .try_init()
                .expect("Failed to initialize tracing subscriber for tests");
        });
        
        // This test will ensure tracing is set up without panicking
        tracing::info!("Test log message from tracing utils");
    }
}