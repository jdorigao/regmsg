use std::env;
use log::{info, debug, warn, error};

mod kmsdrm;
mod wayland;

/// Represents display mode information including width, height, and refresh rate.
#[derive(Debug)]
struct ModeInfo {
    width: i32,
    height: i32,
    vrefresh: i32,
}

/// Detects the current graphics backend (Wayland or KMS/DRM).
///
/// # Returns
/// A string indicating the detected backend: "Wayland" or "KMS/DRM".
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
/// # Arguments
/// * `mode` - A string representing the display mode (e.g., "1920x1080@60").
///
/// # Returns
/// A `Result` containing `ModeInfo` if parsing is successful, or an error message if the format is invalid.
fn parse_mode(mode: &str) -> Result<ModeInfo, Box<dyn std::error::Error>> {
    debug!("Parsing display mode: {}", mode);
    let parts: Vec<&str> = mode.split(&['x', '@'][..]).collect();
    if parts.len() < 2 || parts.len() > 3 {
        error!("Invalid mode format. Expected 'WxH@R' or 'WxH'.");
        return Err("Invalid mode format. Use 'WxH@R' or 'WxH'".into());
    }

    let width = parts[0].parse::<i32>().map_err(|_| "Invalid width")?;
    let height = parts[1].parse::<i32>().map_err(|_| "Invalid height")?;
    let vrefresh = if parts.len() == 3 {
        parts[2].parse::<i32>().map_err(|_| "Invalid refresh rate")?
    } else {
        60 // Default refresh rate
    };

    debug!("Parsed mode: {}x{}@{}", width, height, vrefresh);
    Ok(ModeInfo { width, height, vrefresh })
}

/// Lists available display modes for the specified screen.
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to query.
///
/// # Returns
/// A `Result` containing a string with the list of modes, or an error message.
pub fn list_modes(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    info!("Listing display modes for screen: {:?}", screen);
    match detect_backend() {
        "Wayland" => wayland::wayland_list_modes(screen),
        "KMS/DRM" => kmsdrm::drm_list_modes(),
        _ => {
            warn!("Unknown backend detected.");
            Ok("Unknown backend. Unable to determine display settings.\n".to_string())
        }
    }
}

/// Lists available outputs (e.g., HDMI, VGA).
///
/// # Returns
/// A `Result` containing a string with the list of outputs, or an error message.
pub fn list_outputs() -> Result<String, Box<dyn std::error::Error>> {
    info!("Listing available outputs.");
    match detect_backend() {
        "Wayland" => wayland::wayland_list_outputs(),
        "KMS/DRM" => kmsdrm::drm_list_outputs(),
        _ => {
            warn!("Unknown backend detected.");
            Ok("Unknown backend. Unable to determine display settings.\n".to_string())
        }
    }
}

/// Displays the current display mode for the specified screen.
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to query.
///
/// # Returns
/// A `Result` containing a string with the current mode, or an error message.
pub fn current_mode(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    info!("Fetching current display mode for screen: {:?}", screen);
    match detect_backend() {
        "Wayland" => wayland::wayland_current_mode(screen),
        "KMS/DRM" => kmsdrm::drm_current_mode(),
        _ => {
            warn!("Unknown backend detected.");
            Ok("Unknown backend. Unable to determine display settings.\n".to_string())
        }
    }
}

/// Displays the current output (e.g., HDMI, VGA).
///
/// # Returns
/// A `Result` containing a string with the current output, or an error message.
pub fn current_output() -> Result<String, Box<dyn std::error::Error>> {
    info!("Fetching current output.");
    match detect_backend() {
        "Wayland" => wayland::wayland_current_output(),
        "KMS/DRM" => kmsdrm::drm_current_output(),
        _ => {
            warn!("Unknown backend detected.");
            Ok("Unknown backend. Unable to determine display settings.\n".to_string())
        }
    }
}

/// Displays the current resolution for the specified screen.
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to query.
///
/// # Returns
/// A `Result` containing a string with the current resolution, or an error message.
pub fn current_resolution(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    info!("Fetching current resolution for screen: {:?}", screen);
    match detect_backend() {
        "Wayland" => wayland::wayland_current_resolution(screen),
        "KMS/DRM" => kmsdrm::drm_current_resolution(),
        _ => {
            warn!("Unknown backend detected.");
            Ok("Unknown backend. Unable to determine display settings.\n".to_string())
        }
    }
}

/// Displays the current refresh rate for the specified screen.
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to query.
///
/// # Returns
/// A `Result` containing a string with the current refresh rate, or an error message.
pub fn current_refresh(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    info!("Fetching current refresh rate for screen: {:?}", screen);
    match detect_backend() {
        "Wayland" => wayland::wayland_current_refresh(screen),
        "KMS/DRM" => kmsdrm::drm_current_refresh(),
        _ => {
            warn!("Unknown backend detected.");
            Ok("Unknown backend. Unable to determine display settings.\n".to_string())
        }
    }
}

/// Sets the display mode for the specified screen.
///
/// # Arguments
/// * `screen` - An optional string specifying the screen to configure.
/// * `mode` - A string representing the display mode to set (e.g., "1920x1080@60").
///
/// # Returns
/// A `Result` indicating success or an error message.
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
        let mode_set = parse_mode(mode)?;
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
/// # Arguments
/// * `output` - A string representing the output resolution and refresh rate to set.
///
/// # Returns
/// A `Result` indicating success or an error message.
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
/// # Arguments
/// * `screen` - An optional string specifying the screen to configure.
/// * `rotation` - A string representing the rotation angle (0, 90, 180, or 270 degrees).
///
/// # Returns
/// A `Result` indicating success or an error message.
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
/// # Returns
/// A `Result` indicating success or an error message.
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
/// # Returns
/// A `Result` indicating success or an error message.
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
/// # Arguments
/// * `screen` - An optional string specifying the screen to configure.
///
/// # Returns
/// A `Result` indicating success or an error message.
pub fn min_to_max_resolution(screen: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting resolution to maximum for screen: {:?}", screen);
    // Sets the default maximum resolution to 1920x1080
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
