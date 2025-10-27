//! Display Backend Abstraction
//!
//! This module defines traits and implementations for different display backends
//! (Wayland, DRM/KMS, etc.), enabling a more modular and extensible architecture.

use crate::utils::error::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::debug;

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

    /// Checks if Wayland backend is available by checking for sway socket
    fn is_wayland_available() -> bool {
        Path::new("/var/run/sway-ipc.0.sock").exists()
    }

    /// Checks if KMS/DRM backend is available by checking for DRM devices
    fn is_kms_available() -> bool {
        // Check if any DRM device exists in /dev/dri/ and if we can open them
        if let Ok(entries) = std::fs::read_dir("/dev/dri/") {
            for entry in entries.flatten() {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.starts_with("card") {
                        // Try to use the existing DrmCard::open_available_card method
                        if let Ok(_) = crate::screen::kmsdrm::DrmCard::open_available_card() {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Gets the active backend based on real environment detection
    pub fn get_active_backend(&self) -> Option<&dyn DisplayBackend> {
        // Check backends in order of preference
        for backend in &self.backends {
            let backend_name = backend.backend_name();
            match backend_name {
                "Wayland" => {
                    if Self::is_wayland_available() {
                        debug!("Wayland backend is available and selected");
                        return Some(backend.as_ref());
                    }
                    // If Wayland is not available, continue to next backend
                }
                "KMS/DRM" => {
                    // For KMS/DRM, we just need to verify it's available since it's our fallback
                    if Self::is_kms_available() {
                        debug!("KMS/DRM backend is available and selected as fallback");
                        return Some(backend.as_ref());
                    }
                }
                _ => {
                    // For other backends, return it if it exists
                    // This maintains extensibility for future backends
                    return Some(backend.as_ref());
                }
            }
        }
        None
    }
}
