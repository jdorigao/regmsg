//! Display Backend Abstraction
//!
//! This module defines traits and implementations for different display backends
//! (Wayland, DRM/KMS, etc.), enabling a more modular and extensible architecture.

use crate::utils::error::Result;
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
