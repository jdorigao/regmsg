// Import our new architecture modules
use crate::screen::backend::{BackendManager, DisplayBackend, ModeParams};
use crate::config;
use crate::utils::error::{RegmsgError, Result};

// Modules for backend-specific implementations
pub mod backend;
pub mod kmsdrm;
pub mod wayland;

use tracing::{debug, error, info};

/// Represents display mode information including width, height, and refresh rate.
///
/// This struct is used to store parsed display mode details for further processing, such as
/// setting or querying display configurations.
#[derive(Debug)]
struct ModeInfo {
    width: i32,    // Screen width in pixels
    height: i32,   // Screen height in pixels
    vrefresh: i32, // Refresh rate in Hertz (Hz)
}

/// Service structure that handles all screen operations using the new architecture
pub struct ScreenService {
}

impl ScreenService {
}

/// Parses a display mode string in the format "WxH@R" or "WxH".
///
/// This function splits the input string into components (width, height, and optionally refresh rate)
/// and constructs a `ModeInfo` struct. If the refresh rate is omitted, it defaults to 60 Hz.
///
/// # Arguments
/// * `mode` - A string representing the display mode (e.g., "1920x1080@60").
///
/// # Returns
/// A `Result` containing `ModeInfo` if parsing is successful, or an error message if the format is invalid.
///
/// # Examples
/// ```
/// let mode = parse_mode("1920x1080@60").unwrap();
/// assert_eq!(mode.width, 1920);
/// assert_eq!(mode.height, 1080);
/// assert_eq!(mode.vrefresh, 60);
/// ```
fn parse_mode(mode: &str) -> Result<ModeInfo> {
    debug!("Parsing display mode: {}", mode);
    let parts: Vec<&str> = mode.split(&['x', '@'][..]).collect();
    if parts.len() < 2 || parts.len() > 3 {
        error!("Invalid mode format. Expected 'WxH@R' or 'WxH'.");
        return Err(RegmsgError::InvalidArguments(
            "Invalid mode format. Use 'WxH@R' or 'WxH'".to_string(),
        ));
    }

    // Parse width and height from the split parts
    let width = parts[0]
        .parse::<i32>()
        .map_err(|_| RegmsgError::ParseError("Invalid width".to_string()))?;
    let height = parts[1]
        .parse::<i32>()
        .map_err(|_| RegmsgError::ParseError("Invalid height".to_string()))?;
    // Parse refresh rate if provided, otherwise default to 60 Hz
    let vrefresh = if parts.len() == 3 {
        parts[2]
            .parse::<i32>()
            .map_err(|_| RegmsgError::ParseError("Invalid refresh rate".to_string()))?
    } else {
        60 // Default refresh rate if not specified
    };

    debug!("Parsed mode: {}x{}@{}", width, height, vrefresh);
    Ok(ModeInfo {
        width,
        height,
        vrefresh,
    })
}

/// Lists available display modes for the specified screen.
///
/// This function queries the graphics backend (Wayland or KMS/DRM) to retrieve a list
/// of supported display modes for the given screen. The result is printed to the console
/// and returned as a string.
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to query (e.g., "HDMI-1").
///
/// # Returns
/// A `Result` containing a string with the list of modes, or an error message if the query fails.
pub fn list_modes(screen: Option<&str>) -> Result<String> {
    let backend = ScreenService::default_backend()?;
    let modes = backend.list_modes(screen)?;

    let modes_str = modes
        .iter()
        .map(|mode| {
            format!(
                "{}x{}@{}:{} {}x{}@{}Hz",
                mode.width,
                mode.height,
                mode.refresh_rate,
                mode.name,
                mode.width,
                mode.height,
                mode.refresh_rate
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    Ok(modes_str)
}

/// Lists available outputs (e.g., HDMI, VGA).
///
/// This function retrieves a list of connected display outputs based on the detected graphics backend.
/// The result is printed to the console and returned as a string.
///
/// # Returns
/// A `Result` containing a string with the list of outputs, or an error message if the query fails.
pub fn list_outputs() -> Result<String> {
    let backend = ScreenService::default_backend()?;
    let outputs = backend.list_outputs()?;

    let outputs_str = outputs
        .iter()
        .map(|output| output.name.clone())
        .collect::<Vec<_>>()
        .join("\n");

    Ok(outputs_str)
}

/// Displays the current display mode for the specified screen.
///
/// This function retrieves the active display mode (resolution and refresh rate) for the given screen.
/// The result is printed to the console and returned as a string.
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to query.
///
/// # Returns
/// A `Result` containing a string with the current mode, or an error message if the query fails.
pub fn current_mode(screen: Option<&str>) -> Result<String> {
    let backend = ScreenService::default_backend()?;
    let mode = backend.current_mode(screen)?;

    Ok(format!(
        "{}x{}@{}",
        mode.width, mode.height, mode.refresh_rate
    ))
}

/// Displays the current output (e.g., HDMI, VGA).
///
/// This function identifies the currently active output based on the graphics backend.
/// The result is printed to the console and returned as a string.
///
/// # Returns
/// A `Result` containing a string with the current output, or an error message if the query fails.
pub fn current_output() -> Result<String> {
    let backend = ScreenService::default_backend()?;
    let outputs = backend.list_outputs()?;

    let active_output = outputs
        .iter()
        .find(|output| output.is_connected && output.current_mode.is_some())
        .map(|output| output.name.clone())
        .unwrap_or_else(|| "No active output".to_string());

    Ok(active_output)
}

/// Displays the current resolution for the specified screen.
///
/// This function retrieves the current resolution (width x height) for the given screen.
/// The result is printed to the console and returned as a string.
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to query.
///
/// # Returns
/// A `Result` containing a string with the current resolution, or an error message if the query fails.
pub fn current_resolution(screen: Option<&str>) -> Result<String> {
    let backend = ScreenService::default_backend()?;
    let (width, height) = backend.current_resolution(screen)?;

    Ok(format!("{}x{}", width, height))
}

/// Displays the current refresh rate for the specified screen.
///
/// This function retrieves the current refresh rate (in Hz) for the given screen.
/// The result is printed to the console and returned as a string.
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to query.
///
/// # Returns
/// A `Result` containing a string with the current refresh rate, or an error message if the query fails.
pub fn current_refresh(screen: Option<&str>) -> Result<String> {
    let backend = ScreenService::default_backend()?;
    let refresh_rate = backend.current_refresh_rate(screen)?;

    Ok(format!("{}Hz", refresh_rate))
}

/// Displays the current rotation for the specified screen.
///
/// This function retrieves the current rotation angle (0, 90, 180, or 270 degrees) for the given screen.
/// The result is printed to the console and returned as a string.
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to query.
///
/// # Returns
/// A `Result` containing a string with the current rotation, or an error message if the query fails.
pub fn current_rotation(screen: Option<&str>) -> Result<String> {
    let backend = ScreenService::default_backend()?;
    let rotation = backend.current_rotation(screen)?;

    Ok(rotation.to_string())
}

/// Sets the display mode for the specified screen.
///
/// This function allows setting a specific display mode (resolution and refresh rate) or a maximum resolution
/// indicated by the "max-" prefix. It parses the mode string and delegates to the appropriate backend.
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to configure.
/// * `mode` - A string representing the display mode to set (e.g., "1920x1080@60" or "max-1920x1080").
///
/// # Returns
/// A `Result` indicating success or an error message if the operation fails.
pub fn set_mode(screen: Option<&str>, mode: &str) -> Result<()> {
    let backend = ScreenService::default_backend()?;

    if mode.starts_with("max-") {
        let max_resolution = mode.trim_start_matches("max-");
        backend.set_max_resolution(screen, Some(max_resolution))?;
    } else {
        let mode_info = parse_mode(mode)?;
        let mode_params = ModeParams {
            width: mode_info.width as u32,
            height: mode_info.height as u32,
            refresh_rate: mode_info.vrefresh as u32,
        };
        backend.set_mode(screen, &mode_params)?;
    }
    Ok(())
}

/// Sets the output resolution and refresh rate (e.g., "1920x1080@60").
///
/// This function configures the output settings for the active display using the specified mode string.
///
/// # Arguments
/// * `output` - A string representing the output resolution and refresh rate to set.
///
/// # Returns
/// A `Result` indicating success or an error message if the operation fails.
pub fn set_output(output: &str) -> Result<()> {
    let backend = ScreenService::default_backend()?;
    let mode_info = parse_mode(output)?;
    let mode_params = ModeParams {
        width: mode_info.width as u32,
        height: mode_info.height as u32,
        refresh_rate: mode_info.vrefresh as u32,
    };

    // Apply to all connected outputs without specifying a screen
    backend.set_mode(None, &mode_params)?;
    Ok(())
}

/// Sets the screen rotation for the specified screen.
///
/// This function rotates the display to the specified angle (0, 90, 180, or 270 degrees).
/// The rotation is applied based on the detected graphics backend.
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to configure.
/// * `rotation` - A string representing the rotation angle (0, 90, 180, or 270 degrees).
///
/// # Returns
/// A `Result` indicating success or an error message if the operation fails.
pub fn set_rotation(screen: Option<&str>, rotation: &str) -> Result<()> {
    let backend = ScreenService::default_backend()?;

    // Validate rotation value
    let rotation_value = rotation.parse::<u32>().map_err(|_| {
        RegmsgError::InvalidArguments(format!(
            "Invalid rotation: '{}'. Must be a number",
            rotation
        ))
    })?;

    if ![0, 90, 180, 270].contains(&rotation_value) {
        return Err(RegmsgError::InvalidArguments(
            "Rotation must be one of: 0, 90, 180, 270".to_string(),
        ));
    }

    use crate::screen::backend::RotationParams;
    let rotation_params = RotationParams {
        rotation: rotation_value,
    };

    backend.set_rotation(screen, &rotation_params)?;
    Ok(())
}

/// Takes a screenshot of the current screen.
///
/// This function captures the current screen content and saves it. The implementation
/// depends on the detected backend (Wayland or KMS/DRM).
///
/// # Returns
/// A `Result` indicating success or an error message if the operation fails.
pub fn get_screenshot() -> Result<()> {
    let backend = ScreenService::default_backend()?;

    let filepath = backend.take_screenshot(config::DEFAULT_SCREENSHOT_DIR)?;
    info!("Screenshot saved to: {}", filepath);
    Ok(())
}

/// Maps the touchscreen to the correct display.
///
/// This function configures the touchscreen input to align with the current display.
/// Currently, this is supported only on Wayland.
///
/// # Returns
/// A `Result` indicating success or an error message if the operation fails.
pub fn map_touch_screen() -> Result<()> {
    let backend = ScreenService::default_backend()?;
    backend.map_touchscreen()?;
    Ok(())
}

/// Sets the screen resolution to the maximum supported resolution (e.g., 1920x1080).
///
/// This function sets the display to a predefined maximum resolution, defaulting to 1920x1080
/// if no specific resolution is provided elsewhere.
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to configure.
///
/// # Returns
/// A `Result` indicating success or an error message if the operation fails.
pub fn min_to_max_resolution(screen: Option<&str>) -> Result<()> {
    let backend = ScreenService::default_backend()?;
    // Default maximum resolution
    backend.set_max_resolution(screen, Some(config::DEFAULT_MAX_RESOLUTION))?;
    Ok(())
}

/// Retrieves the currently detected graphics backend.
///
/// This function calls the backend manager and returns the result as a string.
///
/// # Returns
/// A `Result` containing the detected backend as a string ("Wayland" or "KMS/DRM"),
/// or an error message if the operation fails.
pub fn current_backend() -> Result<String> {
    let backend = ScreenService::default_backend()?;
    Ok(backend.backend_name().to_string())
}

impl ScreenService {
    /// Gets a reference to the active backend (helper for current functions)
    fn default_backend() -> Result<&'static dyn DisplayBackend> {
        // For simplicity in this refactoring, we'll return a static reference to a backend
        // In a real implementation, we'd have a singleton or similar pattern
        use std::sync::OnceLock;
        static BACKEND_MANAGER: OnceLock<BackendManager> = OnceLock::new();

        let manager = BACKEND_MANAGER.get_or_init(|| {
            let mut backend_manager = BackendManager::new();
            // Add backends in order of preference

            // Always add both backends - the system will try to use the first available one
            backend_manager.add_backend(Box::new(crate::screen::wayland::WaylandBackend::new()));
            backend_manager.add_backend(Box::new(crate::screen::kmsdrm::DrmBackend::new()));

            backend_manager
        });

        manager
            .get_active_backend()
            .ok_or_else(|| RegmsgError::SystemError("No backend available".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
