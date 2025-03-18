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

pub fn drm_get_modes() -> Result<(), Box<dyn Error>> {
    let card = Card::open_first_available()?;
    for_each_connector(&card, |connector_info| {
        Ok(for mode in connector_info.modes() {
            println!(
                "{}x{}@{}",
                mode.size().0,
                mode.size().1,
                mode.vrefresh()
            );
        })
    })
}

pub fn drm_get_outputs() -> Result<(), Box<dyn Error>> {
    let card = Card::open_first_available()?;
    for_each_connector(&card, |connector_info| {
        Ok(println!(
            "{:?} | {:?}",
            connector_info.interface(),
            connector_info.state()
        ))
    })
}

pub fn drm_current_mode() -> Result<(), Box<dyn Error>> {
    let card = Card::open_first_available()?;
    for_each_connector(&card, |connector_info| -> Result<(), Box<dyn Error>> {
        Ok(if connector_info.state() == connector::State::Connected {
            if let Some(encoder_id) = connector_info.current_encoder() {
                let encoder_info = card.get_encoder(encoder_id)?;
                if let Some(crtc_id) = encoder_info.crtc() {
                    let crtc_info = card.get_crtc(crtc_id)?;
                    if let Some(mode) = crtc_info.mode() {
                        println!(
                            "{}x{}@{}",
                            mode.size().0,
                            mode.size().1,
                            mode.vrefresh()
                        );
                    }
                }
            }
        })
    })
}

pub fn drm_current_output() -> Result<(), Box<dyn Error>> {
    let card = Card::open_first_available()?;
    for_each_connector(&card, |connector_info| {
        Ok(if connector_info.state() == connector::State::Connected {
            println!("{:?}",connector_info.interface());
        })
    })
}

pub fn drm_current_resolution() -> Result<(), Box<dyn Error>> {
    let card = Card::open_first_available()?;
    for_each_connector(&card, |connector_info| -> Result<(), Box<dyn Error>> {
        Ok(if connector_info.state() == connector::State::Connected {
            if let Some(encoder_id) = connector_info.current_encoder() {
                let encoder_info = card.get_encoder(encoder_id)?;
                if let Some(crtc_id) = encoder_info.crtc() {
                    let crtc_info = card.get_crtc(crtc_id)?;
                    if let Some(mode) = crtc_info.mode() {
                        println!(
                            "{}x{}",
                            mode.size().0,
                            mode.size().1
                        );
                    }
                }
            }
        })
    })
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

pub fn drm_current_refresh() -> Result<(), Box<dyn Error>> {
    let card = Card::open_first_available()?;
    for_each_connector(&card, |connector_info| -> Result<(), Box<dyn Error>> {
        Ok(if connector_info.state() == connector::State::Connected {
            if let Some(encoder_id) = connector_info.current_encoder() {
                let encoder_info = card.get_encoder(encoder_id)?;
                if let Some(crtc_id) = encoder_info.crtc() {
                    let crtc_info = card.get_crtc(crtc_id)?;
                    if let Some(mode) = crtc_info.mode() {
                        println!("{}", mode.vrefresh());
                    }
                }
            }
        })
    })
}

pub fn drm_get_screenshot() -> Result<(), Box<dyn std::error::Error>> {
    println!("[TODO] KMS/DRM: Screenshot");
    Ok(())
}
