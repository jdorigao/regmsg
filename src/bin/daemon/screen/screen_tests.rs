// Comprehensive test file for the screen module
// This file contains tests for the screen module's functionality, including KMS/DRM and Wayland backends.

use crate::screen::backend::{
    DisplayBackend, DisplayMode, DisplayOutput, ModeParams, RotationParams,
};
use crate::screen::kmsdrm::DrmBackend;
use crate::screen::parse_mode;
use crate::screen::wayland::WaylandBackend;
use crate::utils::error::RegmsgError;

// Test for DisplayMode serialization and deserialization
#[test]
fn test_display_mode_serialization() {
    let mode = DisplayMode {
        width: 1920,
        height: 1080,
        refresh_rate: 60,
        name: "1920x1080@60".to_string(),
    };

    let serialized = serde_json::to_string(&mode).unwrap();
    let deserialized: DisplayMode = serde_json::from_str(&serialized).unwrap();

    assert_eq!(mode.width, deserialized.width);
    assert_eq!(mode.height, deserialized.height);
    assert_eq!(mode.refresh_rate, deserialized.refresh_rate);
    assert_eq!(mode.name, deserialized.name);
}

// Test for DrmBackend (if possible to instantiate)
#[test]
fn test_drm_backend_creation() {
    let backend = DrmBackend::new();
    assert_eq!(backend.backend_name(), "KMS/DRM");
}

// Test for WaylandBackend (if possible to instantiate)
#[test]
fn test_wayland_backend_creation() {
    let backend = WaylandBackend::new();
    assert_eq!(backend.backend_name(), "Wayland");
}

// Tests for DisplayBackend implementations using mocks
// We'll create structs that implement DisplayBackend to test the logic
struct MockDrmBackend {
    list_outputs_result: Result<Vec<DisplayOutput>, RegmsgError>,
    list_modes_result: Result<Vec<DisplayMode>, RegmsgError>,
    current_mode_result: Result<DisplayMode, RegmsgError>,
    current_rotation_result: Result<u32, RegmsgError>,
}

impl MockDrmBackend {
    fn new() -> Self {
        Self {
            list_outputs_result: Ok(vec![]),
            list_modes_result: Ok(vec![]),
            current_mode_result: Err(RegmsgError::NotFound("Mock mode".to_string())),
            current_rotation_result: Ok(0),
        }
    }
}

impl DisplayBackend for MockDrmBackend {
    fn list_outputs(&self) -> Result<Vec<DisplayOutput>, RegmsgError> {
        match &self.list_outputs_result {
            Ok(outputs) => Ok(outputs.clone()),
            Err(e) => Err(e.clone()),
        }
    }

    fn list_modes(&self, _screen: Option<&str>) -> Result<Vec<DisplayMode>, RegmsgError> {
        match &self.list_modes_result {
            Ok(modes) => Ok(modes.clone()),
            Err(e) => Err(e.clone()),
        }
    }

    fn current_mode(&self, _screen: Option<&str>) -> Result<DisplayMode, RegmsgError> {
        match &self.current_mode_result {
            Ok(mode) => Ok(mode.clone()),
            Err(e) => Err(e.clone()),
        }
    }

    fn current_resolution(&self, _screen: Option<&str>) -> Result<(u32, u32), RegmsgError> {
        match self.current_mode(None) {
            Ok(mode) => Ok((mode.width, mode.height)),
            Err(e) => Err(e),
        }
    }

    fn current_refresh_rate(&self, _screen: Option<&str>) -> Result<u32, RegmsgError> {
        match self.current_mode(None) {
            Ok(mode) => Ok(mode.refresh_rate),
            Err(e) => Err(e),
        }
    }

    fn current_rotation(&self, _screen: Option<&str>) -> Result<u32, RegmsgError> {
        match &self.current_rotation_result {
            Ok(rotation) => Ok(*rotation),
            Err(e) => Err(e.clone()),
        }
    }

    fn set_mode(&self, _screen: Option<&str>, _mode: &ModeParams) -> Result<(), RegmsgError> {
        Ok(())
    }

    fn set_rotation(
        &self,
        _screen: Option<&str>,
        _rotation: &RotationParams,
    ) -> Result<(), RegmsgError> {
        Ok(())
    }

    fn set_max_resolution(
        &self,
        _screen: Option<&str>,
        _max_resolution: Option<&str>,
    ) -> Result<(), RegmsgError> {
        Ok(())
    }

    fn take_screenshot(&self, _screenshot_dir: &str) -> Result<String, RegmsgError> {
        Ok("/tmp/mock_screenshot.png".to_string())
    }

    fn map_touchscreen(&self) -> Result<(), RegmsgError> {
        Ok(())
    }

    fn backend_name(&self) -> &'static str {
        "MockDRM"
    }
}

struct MockWaylandBackend {
    list_outputs_result: Result<Vec<DisplayOutput>, RegmsgError>,
    list_modes_result: Result<Vec<DisplayMode>, RegmsgError>,
    current_mode_result: Result<DisplayMode, RegmsgError>,
    current_rotation_result: Result<u32, RegmsgError>,
}

impl MockWaylandBackend {
    fn new() -> Self {
        Self {
            list_outputs_result: Ok(vec![]),
            list_modes_result: Ok(vec![]),
            current_mode_result: Err(RegmsgError::NotFound("Mock mode".to_string())),
            current_rotation_result: Ok(0),
        }
    }
}

impl DisplayBackend for MockWaylandBackend {
    fn list_outputs(&self) -> Result<Vec<DisplayOutput>, RegmsgError> {
        match &self.list_outputs_result {
            Ok(outputs) => Ok(outputs.clone()),
            Err(e) => Err(e.clone()),
        }
    }

    fn list_modes(&self, _screen: Option<&str>) -> Result<Vec<DisplayMode>, RegmsgError> {
        match &self.list_modes_result {
            Ok(modes) => Ok(modes.clone()),
            Err(e) => Err(e.clone()),
        }
    }

    fn current_mode(&self, _screen: Option<&str>) -> Result<DisplayMode, RegmsgError> {
        match &self.current_mode_result {
            Ok(mode) => Ok(mode.clone()),
            Err(e) => Err(e.clone()),
        }
    }

    fn current_resolution(&self, _screen: Option<&str>) -> Result<(u32, u32), RegmsgError> {
        match self.current_mode(None) {
            Ok(mode) => Ok((mode.width, mode.height)),
            Err(e) => Err(e),
        }
    }

    fn current_refresh_rate(&self, _screen: Option<&str>) -> Result<u32, RegmsgError> {
        match self.current_mode(None) {
            Ok(mode) => Ok(mode.refresh_rate),
            Err(e) => Err(e),
        }
    }

    fn current_rotation(&self, _screen: Option<&str>) -> Result<u32, RegmsgError> {
        match &self.current_rotation_result {
            Ok(rotation) => Ok(*rotation),
            Err(e) => Err(e.clone()),
        }
    }

    fn set_mode(&self, _screen: Option<&str>, _mode: &ModeParams) -> Result<(), RegmsgError> {
        Ok(())
    }

    fn set_rotation(
        &self,
        _screen: Option<&str>,
        _rotation: &RotationParams,
    ) -> Result<(), RegmsgError> {
        Ok(())
    }

    fn set_max_resolution(
        &self,
        _screen: Option<&str>,
        _max_resolution: Option<&str>,
    ) -> Result<(), RegmsgError> {
        Ok(())
    }

    fn take_screenshot(&self, _screenshot_dir: &str) -> Result<String, RegmsgError> {
        Ok("/tmp/mock_screenshot.png".to_string())
    }

    fn map_touchscreen(&self) -> Result<(), RegmsgError> {
        Ok(())
    }

    fn backend_name(&self) -> &'static str {
        "MockWayland"
    }
}

#[test]
fn test_mock_drm_backend_functionality() {
    let mock_backend = MockDrmBackend::new();

    // Tests list_outputs
    let outputs = mock_backend.list_outputs().unwrap();
    assert!(outputs.is_empty());

    // Tests list_modes
    let modes = mock_backend.list_modes(None).unwrap();
    assert!(modes.is_empty());

    // Tests current_mode - should return an error since it's configured this way
    let current_mode_result = mock_backend.current_mode(None);
    assert!(current_mode_result.is_err());

    // Tests current_rotation
    let rotation = mock_backend.current_rotation(None).unwrap();
    assert_eq!(rotation, 0);

    // Tests backend_name
    assert_eq!(mock_backend.backend_name(), "MockDRM");
}

#[test]
fn test_mock_wayland_backend_functionality() {
    let mock_backend = MockWaylandBackend::new();

    // Tests list_outputs
    let outputs = mock_backend.list_outputs().unwrap();
    assert!(outputs.is_empty());

    // Tests list_modes
    let modes = mock_backend.list_modes(None).unwrap();
    assert!(modes.is_empty());

    // Tests current_mode - should return an error since it's configured this way
    let current_mode_result = mock_backend.current_mode(None);
    assert!(current_mode_result.is_err());

    // Tests current_rotation
    let rotation = mock_backend.current_rotation(None).unwrap();
    assert_eq!(rotation, 0);

    // Tests backend_name
    assert_eq!(mock_backend.backend_name(), "MockWayland");
}

// Tests for functions in screen/mod.rs
// These functions use backends, so tests will verify the call logic
// and error handling, but will depend on real backends or mocks.

// #[test]
// fn test_list_modes() {
//     // Example of how to test a specific function
//     // This will depend on the exact implementation and backend configuration
//     let result = crate::screen::list_modes(None);
//     // Check expected result
//     // assert!(result.is_ok());
// }

// #[test]
// fn test_current_mode() {
//     let result = crate::screen::current_mode(None);
//     // Check expected result based on available backends
//     // assert!(result.is_ok());
// }

// #[test]
// fn test_set_mode() {
//     // Tests the set_mode function with a valid mode
//     let result = crate::screen::set_mode(None, "1920x1080@60");
//     // The result may vary depending on the available backend and permissions
//     // We can test error handling or logical success
//     // assert!(result.is_ok() || result.is_err()); // Accepts any result
// }

// #[test]
// fn test_set_rotation() {
//     // Tests the set_rotation function with a valid value
//     let result = crate::screen::set_rotation(None, "90");
//     // The result may vary depending on the backend and permissions
//     // assert!(result.is_ok() || result.is_err()); // Accepts any result
// }

// #[test]
// fn test_current_backend() {
//     // This function depends on the backend detection logic
//     let result = crate::screen::current_backend();
//     // Check if the returned backend is one of the expected ones (Wayland or DRM)
//     // assert!(result.is_ok());
//     // assert!(matches!(result.unwrap().as_str(), "Wayland" | "DRM"));
// }

// #[test]
// fn test_current_output() {
//     let result = crate::screen::current_output();
//     // Check if the result is one of the expected outputs
//     // assert!(result.is_ok());
// }

// Error case tests
#[test]
fn test_invalid_rotation() {
    // Tests the validation of invalid rotation in crate::screen::set_rotation
    let result = crate::screen::set_rotation(None, "45");
    assert!(result.is_err());
    match result {
        Err(RegmsgError::InvalidArguments(_)) => assert!(true),
        _ => assert!(
            false,
            "Expected InvalidArguments error for invalid rotation"
        ),
    }
}

#[test]
fn test_invalid_rotation_non_numeric() {
    // Tests rotation validation with non-numeric value
    let result = crate::screen::set_rotation(None, "abc");
    assert!(result.is_err());
    match result {
        Err(RegmsgError::InvalidArguments(_)) => assert!(true),
        _ => assert!(
            false,
            "Expected InvalidArguments error for non-numeric rotation"
        ),
    }
}

#[test]
fn test_invalid_mode_format() {
    // Tests validation of invalid mode format
    let result = crate::screen::set_mode(None, "invalid_mode");
    assert!(result.is_err());
    match result {
        Err(RegmsgError::InvalidArguments(_)) => assert!(true),
        _ => assert!(
            false,
            "Expected InvalidArguments error for invalid mode format"
        ),
    }
}

#[test]
fn test_invalid_mode_format_special_chars() {
    // Tests validation of mode format with special characters
    let result = crate::screen::set_mode(None, "1920x1080@60@");
    assert!(result.is_err());
    match result {
        Err(RegmsgError::InvalidArguments(_)) => assert!(true),
        _ => assert!(
            false,
            "Expected InvalidArguments error for invalid mode format with special chars"
        ),
    }
}

#[test]
fn test_mode_with_zero_values() {
    // Tests mode validation with zero values
    let result = parse_mode("0x0@0");
    assert!(result.is_ok()); // parse_mode allows zeros, but other parts of the system might reject
    let mode_info = result.unwrap();
    assert_eq!(mode_info.width, 0);
    assert_eq!(mode_info.height, 0);
    assert_eq!(mode_info.vrefresh, 0);
}



// Tests for mode parsing functionality
#[cfg(test)]
mod parse_mode_tests {
    use super::*;
    use crate::screen::parse_mode;

    #[test]
    fn test_parse_mode_with_refresh_rate() {
        let mode_info = parse_mode("1920x1080@60").unwrap();
        assert_eq!(mode_info.width, 1920);
        assert_eq!(mode_info.height, 1080);
        assert_eq!(mode_info.vrefresh, 60);
    }

    #[test]
    fn test_parse_mode_without_refresh_rate() {
        let mode_info = parse_mode("1920x1080").unwrap();
        assert_eq!(mode_info.width, 1920);
        assert_eq!(mode_info.height, 1080);
        assert_eq!(mode_info.vrefresh, 60); // Default value
    }

    #[test]
    fn test_parse_mode_invalid_format() {
        let result = parse_mode("invalid_format");
        assert!(result.is_err());
        match result {
            Err(RegmsgError::InvalidArguments(_)) => assert!(true),
            _ => assert!(false, "Expected InvalidArguments error"),
        }
    }

    #[test]
    fn test_parse_mode_only_width_and_height() {
        let result = parse_mode("800x600");
        assert!(result.is_ok());
        let mode_info = result.unwrap();
        assert_eq!(mode_info.width, 800);
        assert_eq!(mode_info.height, 600);
        assert_eq!(mode_info.vrefresh, 60); // Default value
    }

    #[test]
    fn test_parse_mode_with_high_refresh_rate() {
        let result = parse_mode("1920x1080@144");
        assert!(result.is_ok());
        let mode_info = result.unwrap();
        assert_eq!(mode_info.width, 1920);
        assert_eq!(mode_info.height, 1080);
        assert_eq!(mode_info.vrefresh, 144);
    }
}
