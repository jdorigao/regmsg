use std::env;

mod kmsdrm;
mod wayland;

#[derive(Debug)]
struct ModeInfo {
    width: u32,
    height: u32,
    vrefresh: u32,
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

    let width = parts[0].parse::<u32>()?;
    let height = parts[1].parse::<u32>()?;
    let vrefresh = if parts.len() == 3 {
        parts[2].parse::<u32>()?
    } else {
        60 // Default refresh rate
    };

    Ok(ModeInfo {
        width,
        height,
        vrefresh,
    })
}

pub fn list_modes() {
    if detect_backend() == "Wayland" {
        let _ = wayland::wayland_list_modes();
    } else {
        let _ = kmsdrm::drm_list_modes();
    }

}

pub fn list_outputs() {
    if detect_backend() == "Wayland" {
        let _ = wayland::wayland_list_outputs();
    } else {
        let _ = kmsdrm::drm_list_outputs();
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

pub fn get_display_mode() {
    if detect_backend() == "Wayland" {
        println!("Using backend Wayland.");
    } else {
        println!("Using backend KMS/DRM.");
    }
}

pub fn get_refresh_rate() {
    if detect_backend() == "Wayland" {
        let _ = wayland::wayland_get_refresh_rate();
    } else {
        let _ = kmsdrm::drm_get_refresh_rate();
    }
}

pub fn get_screenshot() {
    if detect_backend() == "Wayland" {
        let _ = wayland::wayland_get_screenshot();
    } else {
        let _ = kmsdrm::drm_get_screenshot();
    }
}
