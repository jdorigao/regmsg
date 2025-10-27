//! Display Backend Abstraction
//!
//! This module defines traits and implementations for different display backends
//! (Wayland, DRM/KMS, etc.), enabling a more modular and extensible architecture.

use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Structure that represents display mode information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
    pub name: String,
}

/// Structure that represents output/device display information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayOutput {
    pub name: String,
    pub modes: Vec<DisplayMode>,
    pub current_mode: Option<DisplayMode>,
    pub is_connected: bool,
    pub rotation: u32,
}

/// Parameters to define a new display mode
#[derive(Debug, Clone)]
pub struct ModeParams {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
}

/// Parameters to configure rotation
#[derive(Debug, Clone)]
pub struct RotationParams {
    pub rotation: u32,
}

/// Central trait for display operations
pub trait DisplayBackend: Send + Sync {
    /// Lists all available display outputs/devices
    fn list_outputs(&self) -> Result<Vec<DisplayOutput>>;

    /// Lists available modes for a specific output
    fn list_modes(&self, screen: Option<&str>) -> Result<Vec<DisplayMode>>;

    /// Gets information about the current mode of a specific output
    fn current_mode(&self, screen: Option<&str>) -> Result<DisplayMode>;

    /// Gets the current resolution of a specific output
    fn current_resolution(&self, screen: Option<&str>) -> Result<(u32, u32)>;

    /// Gets the current refresh rate of a specific output
    fn current_refresh_rate(&self, screen: Option<&str>) -> Result<u32>;

    /// Gets the current rotation of a specific output
    fn current_rotation(&self, screen: Option<&str>) -> Result<u32>;

    /// Sets the display mode for a specific output
    fn set_mode(&self, screen: Option<&str>, mode: &ModeParams) -> Result<()>;

    /// Sets the rotation of a specific output
    fn set_rotation(&self, screen: Option<&str>, rotation: &RotationParams) -> Result<()>;

    /// Sets the maximum resolution for a specific output
    fn set_max_resolution(&self, screen: Option<&str>, max_resolution: Option<&str>) -> Result<()>;

    /// Takes a screenshot
    fn take_screenshot(&self, screenshot_dir: &str) -> Result<String>;

    /// Maps a touchscreen to a specific output
    fn map_touchscreen(&self) -> Result<()>;

    /// Gets the backend name
    fn backend_name(&self) -> &'static str;
}

/// Backend manager that can handle multiple backends
pub struct BackendManager {
    backends: Vec<Box<dyn DisplayBackend>>,
}

impl BackendManager {
    /// Creates a new backend manager
    pub fn new() -> Self {
        Self {
            backends: Vec::new(),
        }
    }

    /// Adds a backend to the manager
    pub fn add_backend(&mut self, backend: Box<dyn DisplayBackend>) {
        self.backends.push(backend);
    }

    /// Gets the active backend (first one that is available and functional)
    pub fn get_active_backend(&self) -> Option<&dyn DisplayBackend> {
        self.backends.first().map(|b| b.as_ref())
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::RegmsgError;

    // Test implementation for the DisplayBackend trait
    struct TestBackend;

    impl DisplayBackend for TestBackend {
        fn list_outputs(&self) -> Result<Vec<DisplayOutput>> {
            Ok(vec![])
        }

        fn list_modes(&self, _screen: Option<&str>) -> Result<Vec<DisplayMode>> {
            Ok(vec![])
        }

        fn current_mode(&self, _screen: Option<&str>) -> Result<DisplayMode> {
            Err(RegmsgError::NotFound("Test mode".to_string()))
        }

        fn current_resolution(&self, _screen: Option<&str>) -> Result<(u32, u32)> {
            Ok((1920, 1080))
        }

        fn current_refresh_rate(&self, _screen: Option<&str>) -> Result<u32> {
            Ok(60)
        }

        fn current_rotation(&self, _screen: Option<&str>) -> Result<u32> {
            Ok(0)
        }

        fn set_mode(&self, _screen: Option<&str>, _mode: &ModeParams) -> Result<()> {
            Ok(())
        }

        fn set_rotation(&self, _screen: Option<&str>, _rotation: &RotationParams) -> Result<()> {
            Ok(())
        }

        fn set_max_resolution(&self, _screen: Option<&str>, _max_resolution: Option<&str>) -> Result<()> {
            Ok(())
        }

        fn take_screenshot(&self, _screenshot_dir: &str) -> Result<String> {
            Ok("/tmp/test.png".to_string())
        }

        fn map_touchscreen(&self) -> Result<()> {
            Ok(())
        }

        fn backend_name(&self) -> &'static str {
            "Test"
        }
    }

    #[test]
    fn test_backend_manager() {
        let mut manager = BackendManager::new();
        manager.add_backend(Box::new(TestBackend));
        
        let active_backend = manager.get_active_backend();
        assert!(active_backend.is_some());
        assert_eq!(active_backend.unwrap().backend_name(), "Test");
    }

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
}