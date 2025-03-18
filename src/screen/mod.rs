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
        "Wayland"
    } else {
        "KMS/DRM"
    }
}

/// Parses a display mode string in the format "WxH@R" or "WxH".
fn parse_mode(mode: &str) -> Result<ModeInfo, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = mode.split(&['x', '@'][..]).collect();
    if parts.len() < 2 || parts.len() > 3 {
        return Err("Invalid mode format. Use 'WxH@R' or 'WxH'".into());
    }

    let width = parts[0].parse::<i32>()?;
    let height = parts[1].parse::<i32>()?;
    let vrefresh = if parts.len() == 3 {
        parts[2].parse::<i32>()?
    } else {
        60 // Default refresh rate
    };

    Ok(ModeInfo {
        width,
        height,
        vrefresh,
    })
}

pub fn get_modes() {
    if detect_backend() == "Wayland" {
        let _ = wayland::wayland_get_modes();
    } else {
        let _ = kmsdrm::drm_get_modes();
    }

}

pub fn get_outputs() {
    if detect_backend() == "Wayland" {
        let _ = wayland::wayland_get_outputs();
    } else {
        let _ = kmsdrm::drm_get_outputs();
    }
}

pub fn current_mode() {
    if detect_backend() == "Wayland" {
        let _ = wayland::wayland_current_mode();
    } else {
        let _ = kmsdrm::drm_current_mode();
    }
}

pub fn current_output() {
    if detect_backend() == "Wayland" {
        let _ = wayland::wayland_current_output();
    } else {
        let _ = kmsdrm::drm_current_output();
    }
}

pub fn current_resolution() {
    if detect_backend() == "Wayland" {
        let _ = wayland::wayland_current_resolution();
    } else {
        let _ = kmsdrm::drm_current_resolution();
    }
}

pub fn set_mode(mode: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mode_set = parse_mode(mode)?;
    if detect_backend() == "Wayland" {
        let _ = wayland::wayland_set_mode(mode_set.width, mode_set.height, mode_set.vrefresh);
    } else {
        let _ = kmsdrm::drm_set_mode(mode_set.width, mode_set.height, mode_set.vrefresh);
    }
    Ok(())
}

pub fn set_output(output: &str) {
    if detect_backend() == "Wayland" {
        let _ = wayland::wayland_set_output(output);
    } else {
        let _ = kmsdrm::drm_set_output(output);
    }
}

pub fn set_rotation(rotation: &str) {
    if detect_backend() == "Wayland" {
        let _ = wayland::wayland_set_rotation(rotation);
    } else {
        let _ = kmsdrm::drm_set_rotation(rotation);
    }
}

pub fn current_refresh() {
    if detect_backend() == "Wayland" {
        let _ = wayland::wayland_current_refresh();
    } else {
        let _ = kmsdrm::drm_current_refresh();
    }
}

pub fn get_screenshot() {
    if detect_backend() == "Wayland" {
        let _ = wayland::wayland_get_screenshot();
    } else {
        let _ = kmsdrm::drm_get_screenshot();
    }
}
