use std::env;

mod kmsdrm;
mod wayland;

// Função auxiliar para detectar o backend gráfico
fn detect_backend() -> &'static str {
    if env::var("WAYLAND_DISPLAY").is_ok() {
        "Wayland"
    } else {
        "KMS/DRM"
    }
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

pub fn set_mode(mode: &str) {
    if detect_backend() == "Wayland" {
        println!("Wayland: Setting display mode to {}...", mode);
    } else {
        println!("KMS/DRM: Setting display mode to {}...", mode);
    }
}

pub fn set_output(output: &str) {
    if detect_backend() == "Wayland" {
        println!("Wayland: Setting output to {}...", output);
    } else {
        println!("KMS/DRM: Setting output to {}...", output);
    }
}

pub fn set_rotation(rotation: &str) {
    if detect_backend() == "Wayland" {
        println!("Wayland: Setting rotation to {}...", rotation);
    } else {
        println!("KMS/DRM: Setting rotation to {}...", rotation);
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
        println!("Wayland: Screenshot");
    } else {
        println!("KMS/DRM: Screenshot");
    }
}

pub fn recorder(recorder: &str) {
    if detect_backend() == "Wayland" {
        println!("Wayland: Setting recording mode to {}...", recorder);
    } else {
        println!("KMS/DRM: Setting recording mode to {}...", recorder);
    }
}
