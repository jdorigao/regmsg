use drm::control::{Device as ControlDevice, connector};
use drm::Device;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::os::unix::io::{AsFd, BorrowedFd};
use std::path::Path;

// Structure representing a DRM device
#[derive(Debug)]
struct Card(File);

impl AsFd for Card {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl Device for Card {}
impl ControlDevice for Card {}

impl Card {
    /// Attempts to open the first available DRM device in `/dev/dri/`.
    ///
    /// # Returns
    /// Returns a `Card` instance or an error if no DRM device is found.
    fn open_first_available() -> Result<Self, Box<dyn Error>> {
        let dri_path = Path::new("/dev/dri/");

        // Iterates over the files in the /dev/dri/ directory
        for entry in dri_path.read_dir()? {
            let entry = entry?;
            let path = entry.path();

            // Checks if the file starts with "card" (card0, card1, ...)
            if let Some(file_name) = path.file_name() {
                if file_name.to_string_lossy().starts_with("card") {
                    let mut options = OpenOptions::new();
                    options.read(true).write(true);

                    if let Ok(file) = options.open(&path) {
                        return Ok(Card(file));
                    }
                }
            }
        }

        Err("No DRM device found in /dev/dri/".into())
    }
}

/// Helper function to iterate over all connectors.
fn for_each_connector<F>(drm_device: &Card, mut f: F) -> Result<(), Box<dyn Error>>
where
    F: FnMut(&connector::Info) -> Result<(), Box<dyn Error>>,
{
    let resources = drm_device.resource_handles()?;
    Ok(for connector in resources.connectors() {
        let connector_info = drm_device.get_connector(*connector, true)?;
        f(&connector_info)?;
    })
}

pub fn drm_list_modes() -> Result<String, Box<dyn Error>> {
    let card = Card::open_first_available()?;
    let mut modes_string = String::new();

    for_each_connector(&card, |connector_info| {
        for mode in connector_info.modes() {
            modes_string.push_str(&format!(
                "{:?}:{}x{}@{}Hz\n",
                mode.name(),
                mode.size().0,
                mode.size().1,
                mode.vrefresh()
            ));
        }
        Ok(())
    })?;

    if modes_string.is_empty() {
        modes_string.push_str("No modes found.\n");
    }
    print!("{}", modes_string);
    Ok(modes_string)
}

pub fn drm_list_outputs() -> Result<String, Box<dyn Error>> {
    let card = Card::open_first_available()?;
    let mut outputs_string = String::new();

    for_each_connector(&card, |connector_info| {
        outputs_string.push_str(&format!(
            "{:?}\n",
            connector_info.interface()
        ));
        Ok(())
    })?;

    if outputs_string.is_empty() {
        outputs_string.push_str("No outputs found.\n");
    }
    print!("{}", outputs_string);
    Ok(outputs_string)
}

pub fn drm_current_mode() -> Result<String, Box<dyn Error>> {
    let card = Card::open_first_available()?;
    let mut current_mode_string = String::new();

    for_each_connector(&card, |connector_info| -> Result<(), Box<dyn Error>> {
        if connector_info.state() == connector::State::Connected {
            if let Some(encoder_id) = connector_info.current_encoder() {
                let encoder_info = card.get_encoder(encoder_id)?;
                if let Some(crtc_id) = encoder_info.crtc() {
                    let crtc_info = card.get_crtc(crtc_id)?;
                    if let Some(mode) = crtc_info.mode() {
                        current_mode_string.push_str(&format!(
                            "{}x{}@{}Hz\n",
                            mode.size().0,
                            mode.size().1,
                            mode.vrefresh()
                        ));
                    }
                }
            }
        }
        Ok(())
    })?;

    if current_mode_string.is_empty() {
        current_mode_string.push_str("No current mode found.\n");
    }
    print!("{}", current_mode_string);
    Ok(current_mode_string)
}


pub fn drm_current_output() -> Result<String, Box<dyn Error>> {
    let card = Card::open_first_available()?;
    let mut current_output_string = String::new();

    for_each_connector(&card, |connector_info| -> Result<(), Box<dyn Error>> {
        if connector_info.state() == connector::State::Connected {
            current_output_string.push_str(&format!(
                "{:?}\n",
                connector_info.interface()
            ));
        }
        Ok(())
    })?;

    if current_output_string.is_empty() {
        current_output_string.push_str("No current output found.\n");
    }
    print!("{}", current_output_string);
    Ok(current_output_string)
}


pub fn drm_current_resolution() -> Result<String, Box<dyn Error>> {
    let card = Card::open_first_available()?;
    let mut current_resolution_string = String::new();

    for_each_connector(&card, |connector_info| -> Result<(), Box<dyn Error>> {
        if connector_info.state() == connector::State::Connected {
            if let Some(encoder_id) = connector_info.current_encoder() {
                let encoder_info = card.get_encoder(encoder_id)?;
                if let Some(crtc_id) = encoder_info.crtc() {
                    let crtc_info = card.get_crtc(crtc_id)?;
                    if let Some(mode) = crtc_info.mode() {
                        current_resolution_string.push_str(&format!(
                            "{}x{}\n",
                            mode.size().0,
                            mode.size().1
                        ));
                    }
                }
            }
        }
        Ok(())
    })?;

    if current_resolution_string.is_empty() {
        current_resolution_string.push_str("No current resolution found.\n");
    }
    print!("{}", current_resolution_string);
    Ok(current_resolution_string)
}

pub fn drm_current_refresh() -> Result<String, Box<dyn Error>> {
    let card = Card::open_first_available()?;
    let mut current_refresh_string = String::new();

    for_each_connector(&card, |connector_info| -> Result<(), Box<dyn Error>> {
        if connector_info.state() == connector::State::Connected {
            if let Some(encoder_id) = connector_info.current_encoder() {
                let encoder_info = card.get_encoder(encoder_id)?;
                if let Some(crtc_id) = encoder_info.crtc() {
                    let crtc_info = card.get_crtc(crtc_id)?;
                    if let Some(mode) = crtc_info.mode() {
                        current_refresh_string.push_str(&format!(
                            "{}Hz\n",
                            mode.vrefresh()
                        ));
                    }
                }
            }
        }
        Ok(())
    })?;

    if current_refresh_string.is_empty() {
        current_refresh_string.push_str("No current refresh rate found.\n");
    }
    print!("{}", current_refresh_string);
    Ok(current_refresh_string)
}


pub fn drm_set_mode(width: i32, height: i32, vrefresh: i32) -> Result<(), Box<dyn Error>> {
    println!(
        "[TODO] KMS/DRM: Setting display mode to {}x{}@{}...",
        width, height, vrefresh
    );
    Ok(())
}

pub fn drm_set_output(output: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("[TODO] KMS/DRM: Setting output for {}...", output);
    Ok(())
}

pub fn drm_set_rotation(rotation: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("[TODO] KMS/DRM: Setting rotation for {}...", rotation);
    Ok(())
}

pub fn drm_get_screenshot() -> Result<(), Box<dyn std::error::Error>> {
    println!("[TODO] KMS/DRM: Screenshot");
    Ok(())
}

pub fn drm_to_max_resolution(max_resolution: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("[TODO] KMS/DRM: minTomaxResolution {:?}", max_resolution);
    Ok(())
}
