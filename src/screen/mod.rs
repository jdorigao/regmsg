use std::env;
use log::{info, debug, warn, error};

mod kmsdrm;
mod wayland;

/// Represents display mode information including width, height, and refresh rate.
///
/// This struct is used to store parsed display mode details for further processing.
#[derive(Debug)]
struct ModeInfo {
    width: i32,    // Screen width in pixels
    height: i32,   // Screen height in pixels
    vrefresh: i32, // Refresh rate in Hertz (Hz)
}

/// Detects the current graphics backend (Wayland or KMS/DRM).
///
/// This function checks for the presence of the `WAYLAND_DISPLAY` environment variable
/// to determine if the system is running Wayland. If not, it defaults to KMS/DRM.
///
/// # Returns
/// A static string indicating the detected backend: "Wayland" or "KMS/DRM".
fn detect_backend() -> &'static str {
    if env::var("WAYLAND_DISPLAY").is_ok() {
        info!("Detected Wayland backend.");
        return "Wayland";
    } else {
        info!("Detected KMS/DRM backend.");
        return "KMS/DRM";
    }
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
fn parse_mode(mode: &str) -> Result<ModeInfo, Box<dyn std::error::Error>> {
    debug!("Parsing display mode: {}", mode);
    let parts: Vec<&str> = mode.split(&['x', '@'][..]).collect();
    if parts.len() < 2 || parts.len() > 3 {
        error!("Invalid mode format. Expected 'WxH@R' or 'WxH'.");
        return Err("Invalid mode format. Use 'WxH@R' or 'WxH'".into());
    }

    // Parse width and height from the split parts
    let width = parts[0].parse::<i32>().map_err(|_| "Invalid width")?;
    let height = parts[1].parse::<i32>().map_err(|_| "Invalid height")?;
    let vrefresh = if parts.len() == 3 {
        parts[2].parse::<i32>().map_err(|_| "Invalid refresh rate")?
    } else {
        60 // Default refresh rate if not specified
    };

    debug!("Parsed mode: {}x{}@{}", width, height, vrefresh);
    Ok(ModeInfo { width, height, vrefresh })
}

/// Lists available display modes for the specified screen.
///
/// This function queries the graphics backend (Wayland or KMS/DRM) to retrieve a list
/// of supported display modes for the given screen.
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to query (e.g., "HDMI-1").
///
/// # Returns
/// A `Result` containing a string with the list of modes, or an error message if the query fails.
pub fn list_modes(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    info!("Listing display modes for screen: {:?}", screen);
    let result = match detect_backend() {
        "Wayland" => wayland::wayland_list_modes(screen), // Call Wayland-specific function
        "KMS/DRM" => kmsdrm::drm_list_modes(),            // Call KMS/DRM-specific function
        _ => {
            warn!("Unknown backend detected.");
            Ok("Unknown backend. Unable to determine display settings.\n".to_string())
        }
    }?;
    println!("{}", result); // Output the result to the console
    Ok(result)
}

/// Lists available outputs (e.g., HDMI, VGA).
///
/// This function retrieves a list of connected display outputs based on the detected graphics backend.
///
/// # Returns
/// A `Result` containing a string with the list of outputs, or an error message if the query fails.
pub fn list_outputs() -> Result<String, Box<dyn std::error::Error>> {
    info!("Listing available outputs.");
    let result = match detect_backend() {
        "Wayland" => wayland::wayland_list_outputs(),
        "KMS/DRM" => kmsdrm::drm_list_outputs(),
        _ => {
            warn!("Unknown backend detected.");
            Ok("Unknown backend. Unable to determine display settings.\n".to_string())
        }
    }?;
    println!("{}", result);
    Ok(result)
}

/// Displays the current display mode for the specified screen.
///
/// This function retrieves the active display mode (resolution and refresh rate) for the given screen.
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to query.
///
/// # Returns
/// A `Result` containing a string with the current mode, or an error message if the query fails.
pub fn current_mode(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    info!("Fetching current display mode for screen: {:?}", screen);
    let result = match detect_backend() {
        "Wayland" => wayland::wayland_current_mode(screen),
        "KMS/DRM" => kmsdrm::drm_current_mode(),
        _ => {
            warn!("Unknown backend detected.");
            Ok("Unknown backend. Unable to determine display settings.\n".to_string())
        }
    }?;
    println!("{}", result);
    Ok(result)
}

/// Displays the current output (e.g., HDMI, VGA).
///
/// This function identifies the currently active output based on the graphics backend.
///
/// # Returns
/// A `Result` containing a string with the current output, or an error message if the query fails.
pub fn current_output() -> Result<String, Box<dyn std::error::Error>> {
    info!("Fetching current output.");
    let result = match detect_backend() {
        "Wayland" => wayland::wayland_current_output(),
        "KMS/DRM" => kmsdrm::drm_current_output(),
        _ => {
            warn!("Unknown backend detected.");
            Ok("Unknown backend. Unable to determine display settings.\n".to_string())
        }
    }?;
    println!("{}", result);
    Ok(result)
}

/// Displays the current resolution for the specified screen.
///
/// This function retrieves the current resolution (width x height) for the given screen.
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to query.
///
/// # Returns
/// A `Result` containing a string with the current resolution, or an error message if the query fails.
pub fn current_resolution(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    info!("Fetching current resolution for screen: {:?}", screen);
    let result = match detect_backend() {
        "Wayland" => wayland::wayland_current_resolution(screen),
        "KMS/DRM" => kmsdrm::drm_current_resolution(),
        _ => {
            warn!("Unknown backend detected.");
            Ok("Unknown backend. Unable to determine display settings.\n".to_string())
        }
    }?;
    println!("{}", result);
    Ok(result)
}

/// Displays the current refresh rate for the specified screen.
///
/// This function retrieves the current refresh rate (in Hz) for the given screen.
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to query.
///
/// # Returns
/// A `Result` containing a string with the current refresh rate, or an error message if the query fails.
pub fn current_refresh(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    info!("Fetching current refresh rate for screen: {:?}", screen);
    let result = match detect_backend() {
        "Wayland" => wayland::wayland_current_refresh(screen),
        "KMS/DRM" => kmsdrm::drm_current_refresh(),
        _ => {
            warn!("Unknown backend detected.");
            Ok("Unknown backend. Unable to determine display settings.\n".to_string())
        }
    }?;
    println!("{}", result);
    Ok(result)
}

/// Sets the display mode for the specified screen.
///
/// This function allows setting a specific display mode (resolution and refresh rate) or a maximum resolution
/// indicated by the "max-" prefix.
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to configure.
/// * `mode` - A string representing the display mode to set (e.g., "1920x1080@60" or "max-1920x1080").
///
/// # Returns
/// A `Result` indicating success or an error message if the operation fails.
pub fn set_mode(screen: Option<&str>, mode: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting display mode for screen: {:?} to {}", screen, mode);
    if mode.starts_with("max-") {
        let max_resolution = mode.trim_start_matches("max-").to_string();
        match detect_backend() {
            "Wayland" => wayland::wayland_min_to_max_resolution(screen, Some(max_resolution))?,
            "KMS/DRM" => kmsdrm::drm_to_max_resolution(Some(max_resolution))?,
            _ => {
                warn!("Unknown backend detected.");
                println!("Unknown backend. Unable to determine display settings.");
            }
        }
    } else {
        let mode_set = parse_mode(mode)?; // Parse the mode string into components
        match detect_backend() {
            "Wayland" => wayland::wayland_set_mode(screen, mode_set.width, mode_set.height, mode_set.vrefresh)?,
            "KMS/DRM" => kmsdrm::drm_set_mode(mode_set.width, mode_set.height, mode_set.vrefresh)?,
            _ => {
                warn!("Unknown backend detected.");
                println!("Unknown backend. Unable to determine display settings.");
            }
        }
    }
    Ok(())
}

/// Sets the output resolution and refresh rate (e.g., "1920x1080@60").
///
/// This function configures the output settings for the active display.
///
/// # Arguments
/// * `output` - A string representing the output resolution and refresh rate to set.
///
/// # Returns
/// A `Result` indicating success or an error message if the operation fails.
pub fn set_output(output: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting output to: {}", output);
    match detect_backend() {
        "Wayland" => wayland::wayland_set_output(output)?,
        "KMS/DRM" => kmsdrm::drm_set_output(output)?,
        _ => {
            warn!("Unknown backend detected.");
            println!("Unknown backend. Unable to determine display settings.");
        }
    }
    Ok(())
}

/// Sets the screen rotation for the specified screen.
///
/// This function rotates the display to the specified angle (0, 90, 180, or 270 degrees).
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to configure.
/// * `rotation` - A string representing the rotation angle (0, 90, 180, or 270 degrees).
///
/// # Returns
/// A `Result` indicating success or an error message if the operation fails.
pub fn set_rotation(screen: Option<&str>, rotation: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting rotation for screen: {:?} to {}", screen, rotation);
    match detect_backend() {
        "Wayland" => wayland::wayland_set_rotation(screen, rotation)?,
        "KMS/DRM" => kmsdrm::drm_set_rotation(rotation)?,
        _ => {
            warn!("Unknown backend detected.");
            println!("Unknown backend. Unable to determine display settings.");
        }
    }
    Ok(())
}

/// Takes a screenshot of the current screen.
///
/// This function captures the current screen content and saves it (implementation depends on the backend).
///
/// # Returns
/// A `Result` indicating success or an error message if the operation fails.
pub fn get_screenshot() -> Result<(), Box<dyn std::error::Error>> {
    info!("Taking screenshot.");
    match detect_backend() {
        "Wayland" => wayland::wayland_get_screenshot()?,
        "KMS/DRM" => kmsdrm::drm_get_screenshot()?,
        _ => {
            warn!("Unknown backend detected.");
            println!("Unknown backend. Unable to determine display settings.");
        }
    }
    Ok(())
}

/// Maps the touchscreen to the correct display.
///
/// This function configures the touchscreen input to align with the current display (Wayland only).
///
/// # Returns
/// A `Result` indicating success or an error message if the operation fails.
pub fn map_touch_screen() -> Result<(), Box<dyn std::error::Error>> {
    info!("Mapping touchscreen.");
    match detect_backend() {
        "Wayland" => wayland::wayland_map_touch_screen()?,
        "KMS/DRM" => {
            warn!("No touchscreen support for KMS/DRM.");
            println!("No touchscreen support.");
        }
        _ => {
            warn!("Unknown backend detected.");
            println!("Unknown backend. Unable to determine display settings.");
        }
    }
    Ok(())
}

/// Sets the screen resolution to the maximum supported resolution (e.g., 1920x1080).
///
/// This function sets the display to a predefined maximum resolution, defaulting to 1920x1080.
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to configure.
///
/// # Returns
/// A `Result` indicating success or an error message if the operation fails.
pub fn min_to_max_resolution(screen: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting resolution to maximum for screen: {:?}", screen);
    // Default maximum resolution if not overridden elsewhere
    let max_resolution = "1920x1080".to_string();

    match detect_backend() {
        "Wayland" => wayland::wayland_min_to_max_resolution(screen, Some(max_resolution))?,
        "KMS/DRM" => kmsdrm::drm_to_max_resolution(Some(max_resolution))?,
        _ => {
            warn!("Unknown backend detected.");
            println!("Unknown backend. Unable to determine display settings.");
        }
    }
    Ok(())
}
