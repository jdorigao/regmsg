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
    for connector in resources.connectors() {
        let connector_info = drm_device.get_connector(*connector, true)?;
        f(&connector_info)?;
    }
    Ok(())
}

/// Lists all available display modes for each connector.
pub fn drm_list_modes() -> Result<(), Box<dyn Error>> {
    let card = Card::open_first_available()?;
    for_each_connector(&card, |connector_info| {
        for mode in connector_info.modes() {
            println!(
                "{}x{}@{}",
                mode.size().0,
                mode.size().1,
                mode.vrefresh()
            );
        }
        Ok(())
    })
}

/// Lists all connectors and their states.
pub fn drm_list_outputs() -> Result<(), Box<dyn Error>> {
    let card = Card::open_first_available()?;
    for_each_connector(&card, |connector_info| {
        println!(
            "{:?} | {:?}",
            connector_info.interface(),
            connector_info.state()
        );
        Ok(())
    })
}

/// Lists the active display mode for each connected connector.
pub fn drm_current_mode() -> Result<(), Box<dyn Error>> {
    let card = Card::open_first_available()?;
    for_each_connector(&card, |connector_info| {
        if connector_info.state() == connector::State::Connected {
            if let Some(encoder_id) = connector_info.current_encoder() {
                let encoder_info = card.get_encoder(encoder_id)?;
                if let Some(crtc_id) = encoder_info.crtc() {
                    let crtc_info = card.get_crtc(crtc_id)?;
                    if let Some(mode) = crtc_info.mode() {
                        println!(
                            "{:?} | {}x{}@{}",
                            connector_info.interface(),
                            mode.size().0,
                            mode.size().1,
                            mode.vrefresh()
                        );
                        return Ok(());
                    }
                }
            }
            Err("No active mode found".into())
        } else {
            println!("{:?} | Disconnected", connector_info.interface());
            Ok(())
        }
    })
}
