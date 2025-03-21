use std::env;

mod kmsdrm;
mod wayland;

#[derive(Debug)]
struct ModeInfo {
    width: i32,
    height: i32,
    vrefresh: i32,
}

// Função auxiliar para detectar o backend gráfico
fn detect_backend() -> &'static str {
    if env::var("WAYLAND_DISPLAY").is_ok() {
        return "Wayland"
    } else {
        return "KMS/DRM"
    }
}

/// Parses a display mode string in the format "WxH@R" or "WxH".
fn parse_mode(mode: &str) -> Result<ModeInfo, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = mode.split(&['x', '@'][..]).collect();
    if parts.len() < 2 || parts.len() > 3 {
        return Err("Invalid mode format. Use 'WxH@R' or 'WxH'".into());
    }

    let width = parts[0].parse::<i32>().map_err(|_| "Invalid width")?;
    let height = parts[1].parse::<i32>().map_err(|_| "Invalid height")?;
    let vrefresh = if parts.len() == 3 {
        parts[2].parse::<i32>().map_err(|_| "Invalid refresh rate")?
    } else {
        60 // Default refresh rate
    };

    Ok(ModeInfo { width, height, vrefresh })
}

pub fn list_modes() -> Result<String, Box<dyn std::error::Error>> {
    match detect_backend() {
        "Wayland" => wayland::wayland_list_modes(),
        "KMS/DRM" => kmsdrm::drm_list_modes(),
        _ => Ok("Unknown backend. Unable to determine display settings.\n".to_string()),
    }
}

pub fn list_outputs() -> Result<String, Box<dyn std::error::Error>> {
    match detect_backend() {
        "Wayland" => wayland::wayland_list_outputs(),
        "KMS/DRM" => kmsdrm::drm_list_outputs(),
        _ => Ok("Unknown backend. Unable to determine display settings.\n".to_string()),
    }
}

pub fn current_mode() -> Result<String, Box<dyn std::error::Error>> {
    match detect_backend() {
        "Wayland" => wayland::wayland_current_mode(),
        "KMS/DRM" => kmsdrm::drm_current_mode(),
        _ => Ok("Unknown backend. Unable to determine display settings.\n".to_string()),
    }
}

pub fn current_output() -> Result<String, Box<dyn std::error::Error>> {
    match detect_backend() {
        "Wayland" => wayland::wayland_current_output(),
        "KMS/DRM" => kmsdrm::drm_current_output(),
        _ => Ok("Unknown backend. Unable to determine display settings.\n".to_string()),
    }
}

pub fn current_resolution() -> Result<String, Box<dyn std::error::Error>> {
    match detect_backend() {
        "Wayland" => wayland::wayland_current_resolution(),
        "KMS/DRM" => kmsdrm::drm_current_resolution(),
        _ => Ok("Unknown backend. Unable to determine display settings.\n".to_string()),
    }
}

pub fn current_refresh() -> Result<String, Box<dyn std::error::Error>> {
    match detect_backend() {
        "Wayland" => wayland::wayland_current_refresh(),
        "KMS/DRM" => kmsdrm::drm_current_refresh(),
        _ => Ok("Unknown backend. Unable to determine display settings.\n".to_string()),
    }
}

pub fn set_mode(mode: &str) -> Result<(), Box<dyn std::error::Error>> {
    if mode.starts_with("max-") {
        let max_resolution = mode.trim_start_matches("max-").to_string();
        match detect_backend() {
            "Wayland" => wayland::wayland_min_to_max_resolution(Some(max_resolution))?,
            "KMS/DRM" => kmsdrm::drm_to_max_resolution(Some(max_resolution))?,
            _ => println!("Unknown backend. Unable to determine display settings."),
        }
    } else {
        let mode_set = parse_mode(mode)?;
        match detect_backend() {
            "Wayland" => wayland::wayland_set_mode(mode_set.width, mode_set.height, mode_set.vrefresh)?,
            "KMS/DRM" => kmsdrm::drm_set_mode(mode_set.width, mode_set.height, mode_set.vrefresh)?,
            _ => println!("Unknown backend. Unable to determine display settings."),
        }
    }
    Ok(())
}

pub fn set_output(output: &str) -> Result<(), Box<dyn std::error::Error>> {
    match detect_backend() {
        "Wayland" => wayland::wayland_set_output(output)?,
        "KMS/DRM" => kmsdrm::drm_set_output(output)?,
        _ => println!("Unknown backend. Unable to determine display settings."),
    }
    Ok(())
}

pub fn set_rotation(rotation: &str) -> Result<(), Box<dyn std::error::Error>> {
    match detect_backend() {
        "Wayland" => wayland::wayland_set_rotation(rotation)?,
        "KMS/DRM" => kmsdrm::drm_set_rotation(rotation)?,
        _ => println!("Unknown backend. Unable to determine display settings."),
    }
    Ok(())
}

pub fn get_screenshot() -> Result<(), Box<dyn std::error::Error>> {
    match detect_backend() {
        "Wayland" => wayland::wayland_get_screenshot()?,
        "KMS/DRM" => kmsdrm::drm_get_screenshot()?,
        _ => println!("Unknown backend. Unable to determine display settings."),
    }
    Ok(())
}

pub fn map_touch_screen() -> Result<(), Box<dyn std::error::Error>> {
    match detect_backend() {
        "Wayland" => wayland::wayland_map_touch_screen()?,
        "KMS/DRM" => println!("No touchscreen support."),
        _ => println!("Unknown backend. Unable to determine display settings."),
    }
    Ok(())
}

pub fn min_to_max_resolution() -> Result<(), Box<dyn std::error::Error>> {
    // Sets the default maximum resolution to 1920x1080
    let max_resolution = "1920x1080".to_string();

    match detect_backend() {
        "Wayland" => wayland::wayland_min_to_max_resolution(Some(max_resolution))?,
        "KMS/DRM" => kmsdrm::drm_to_max_resolution(Some(max_resolution))?,
        _ => println!("Unknown backend. Unable to determine display settings."),
    }
    Ok(())
}
