use drm::control::{Device as ControlDevice, connector};
use drm::Device;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::os::unix::io::{AsFd, BorrowedFd};
use std::path::Path;
use log::{info, error, debug, warn};

/// Represents a DRM (Direct Rendering Manager) device card.
///
/// This struct wraps a `File` object representing an opened DRM device file (e.g., `/dev/dri/card0`).
#[derive(Debug)]
struct Card(File);

/// Implements the `AsFd` trait to allow `Card` to provide a file descriptor.
impl AsFd for Card {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

/// Implements the `Device` trait for `Card`, enabling basic DRM operations.
impl Device for Card {}

/// Implements the `ControlDevice` trait for `Card`, enabling control-specific DRM operations.
impl ControlDevice for Card {}

impl Card {
    fn open_first_available() -> Result<Self, Box<dyn Error>> {
        let dri_path = Path::new("/dev/dri/");
        info!("Searching for DRM devices in {:?}", dri_path);

        debug!("Reading directory entries in {:?}", dri_path);
        for entry in dri_path.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if let Some(file_name) = path.file_name() {
                if file_name.to_string_lossy().starts_with("card") {
                    debug!("Trying to open device: {:?}", path);
                    let mut options = OpenOptions::new();
                    options.read(true).write(true);
                    match options.open(&path) {
                        Ok(file) => {
                            info!("Successfully opened DRM device: {:?}", path);
                            debug!("File descriptor obtained for {:?}", path);
                            return Ok(Card(file));
                        }
                        Err(e) => warn!("Failed to open {:?}: {}", path, e),
                    }
                }
            }
        }
        error!("No DRM device found in /dev/dri/");
        Err("No DRM device found in /dev/dri/".into())
    }
}

/// Iterates over all available connectors of the DRM device, applying a given function.
///
/// # Arguments
/// - `drm_device`: A reference to the `Card` representing the DRM device.
/// - `screen`: An optional screen name to filter connectors (e.g., "HDMI", "DP").
/// - `f`: A function that is executed for each connector found.
///
/// # Returns
/// - `Ok(())` if iteration completes successfully.
/// - `Err(Box<dyn Error>)` if any error occurs during connector processing.
fn for_each_connector<F>(drm_device: &Card, screen: Option<&str>, mut f: F) -> Result<(), Box<dyn Error>>
where
    F: FnMut(&connector::Info) -> Result<(), Box<dyn Error>>,
{
    debug!("Fetching resource handles for DRM device");

    // Retrieve all connectors once
    let resources = drm_device.resource_handles()?;
    let connectors = resources.connectors();

    debug!("Found {} connectors", connectors.len());

    connectors.iter()
        .filter_map(|&connector_handle| {
            // Fetch connector info; return None if failed (prevents unnecessary processing)
            match drm_device.get_connector(connector_handle, true) {
                Ok(connector_info) => Some(connector_info),
                Err(e) => {
                    warn!("Failed to get info for connector {:?}: {}", connector_handle, e);
                    None
                }
            }
        })
        .filter(|connector_info| {
            // If a screen filter is specified, check if the connector matches using direct comparison
            match screen {
                Some(screen_name) if format!("{:?}", connector_info.interface()) != screen_name => {
                    debug!(
                        "Skipping connector {:?}-{:?} - doesn't match screen {}",
                        connector_info.interface(),
                        connector_info.interface_id(),
                        screen_name
                    );
                    false
                }
                _ => true, // Process all connectors if no filter is applied
            }
        })
        .try_for_each(|connector_info| f(&connector_info))?; // Apply function `f` to each valid connector

    debug!("Connector iteration completed successfully");
    Ok(())
}

/// Lists all available display modes for a specified screen.
///
/// # Arguments
/// - `screen`: Optional screen name to filter modes (e.g., "HDMI", "DP").
///
/// # Returns
/// - A string containing all available modes, including hardcoded maximums and connector-specific modes.
pub fn drm_list_modes(screen: Option<&str>) -> Result<String, Box<dyn Error>> {
    info!("Listing display modes for screen: {:?}", screen);
    debug!("Opening first available DRM card");
    let card = Card::open_first_available()?;
    let mut modes_string = String::new();

    // Add hardcoded maximum resolutions
    debug!("Adding hardcoded maximum resolutions");
    modes_string.push_str(&format!("{}", "max-1920x1080:maximum 1920x1080"));
    modes_string.push_str(&format!("\n{}", "max-640x480:maximum 640x480"));

    debug!("Iterating over connectors to list modes");
    for_each_connector(&card, screen, |connector_info| {
        debug!("Processing connector {:?}-{:?}", connector_info.interface(), connector_info.interface_id());
        let modes = connector_info.modes();
        debug!("Found {} modes for connector", modes.len());

        for mode in modes {
            let mode_str = format!(
                "\n{}x{}:{}x{}@{}Hz",
                mode.size().0,
                mode.size().1,
                mode.size().0,
                mode.size().1,
                mode.vrefresh()
            );
            debug!("Adding mode: {}", mode_str.trim());
            modes_string.push_str(&mode_str);
        }
        Ok(())
    })?;

    if modes_string.is_empty() {
        warn!("No display modes found for screen: {:?}", screen);
        modes_string.push_str("No modes found.");
    } else {
        info!("Successfully listed display modes for screen: {:?}", screen);
    }
    Ok(modes_string)
}

/// Lists all available outputs (connectors) on the DRM device.
///
/// # Returns
/// - A string listing all connector names (e.g., "HDMI-A-1", "DP-1").
pub fn drm_list_outputs() -> Result<String, Box<dyn Error>> {
    info!("Listing available outputs");
    debug!("Opening first available DRM card");
    let card = Card::open_first_available()?;
    let mut outputs_string = String::new();
    let mut first = true; // Flag to control line breaks

    debug!("Iterating over connectors to list outputs");
    for_each_connector(&card, None, |connector_info| {
        let output_str = format!("{:?}-{:?}", connector_info.interface(), connector_info.interface_id());
        debug!("Found output: {}", output_str);

        if !first {
            outputs_string.push_str("\n"); // Add newline before all but the first output
        }
        outputs_string.push_str(&output_str);
        first = false;
        Ok(())
    })?;

    if outputs_string.is_empty() {
        warn!("No outputs found");
        outputs_string.push_str("No outputs found.");
    } else {
        info!("Successfully listed outputs");
    }
    Ok(outputs_string)
}

/// Retrieves the current display mode for a specified screen.
///
/// # Arguments
/// - `screen`: Optional screen name to filter the mode.
///
/// # Returns
/// - A string in the format "WIDTHxHEIGHT@REFRESH_RATE" (e.g., "1920x1080@60Hz").
pub fn drm_current_mode(screen: Option<&str>) -> Result<String, Box<dyn Error>> {
    info!("Getting current mode for screen: {:?}", screen);
    debug!("Opening first available DRM card");
    let card = Card::open_first_available()?;
    let mut current_mode_string = String::new();

    debug!("Iterating over connectors to find current mode");
    for_each_connector(&card, screen, |connector_info| -> Result<(), Box<dyn Error>> {
        if connector_info.state() == connector::State::Connected {
            debug!("Checking connected connector {:?}-{:?}", connector_info.interface(), connector_info.interface_id());
            if let Some(encoder_id) = connector_info.current_encoder() {
                debug!("Fetching encoder info for ID {:?}", encoder_id);
                let encoder_info = card.get_encoder(encoder_id)?;
                if let Some(crtc_id) = encoder_info.crtc() {
                    debug!("Fetching CRTC info for ID {:?}", crtc_id);
                    let crtc_info = card.get_crtc(crtc_id)?;
                    if let Some(mode) = crtc_info.mode() {
                        let mode_str = format!(
                            "{}x{}@{}Hz",
                            mode.size().0,
                            mode.size().1,
                            mode.vrefresh()
                        );
                        debug!("Current mode found: {}", mode_str);
                        current_mode_string.push_str(&mode_str);
                    } else {
                        debug!("No mode found for CRTC {:?}", crtc_id);
                    }
                }
            }
        }
        Ok(())
    })?;

    if current_mode_string.is_empty() {
        warn!("No current mode found for screen: {:?}", screen);
        current_mode_string.push_str("No current mode found.");
    } else {
        info!("Current mode retrieved: {}", current_mode_string);
    }
    Ok(current_mode_string)
}

/// Retrieves the currently active output (connected connector).
///
/// # Returns
/// - A string with the name of the connected output (e.g., "HDMI-A-1").
pub fn drm_current_output() -> Result<String, Box<dyn Error>> {
    info!("Getting current output");
    debug!("Opening first available DRM card");
    let card = Card::open_first_available()?;
    let mut current_output_string = String::new();

    debug!("Iterating over connectors to find current output");
    for_each_connector(&card, None, |connector_info| -> Result<(), Box<dyn Error>> {
        if connector_info.state() == connector::State::Connected {
            let output_str = format!("{:?}-{:?}", connector_info.interface(), connector_info.interface_id());
            debug!("Found connected output: {}", output_str);
            current_output_string.push_str(&output_str);
        }
        Ok(())
    })?;

    if current_output_string.is_empty() {
        warn!("No current output found");
        current_output_string.push_str("No current output found.");
    } else {
        info!("Current output retrieved: {}", current_output_string);
    }
    Ok(current_output_string)
}

/// Retrieves the current resolution for a specified screen.
///
/// # Arguments
/// - `screen`: Optional screen name to filter the resolution.
///
/// # Returns
/// - A string in the format "WIDTHxHEIGHT" (e.g., "1920x1080").
pub fn drm_current_resolution(screen: Option<&str>) -> Result<String, Box<dyn Error>> {
    info!("Getting current resolution for screen: {:?}", screen);
    debug!("Opening first available DRM card");
    let card = Card::open_first_available()?;
    let mut current_resolution_string = String::new();

    debug!("Iterating over connectors to find current resolution");
    for_each_connector(&card, screen, |connector_info| -> Result<(), Box<dyn Error>> {
        if connector_info.state() == connector::State::Connected {
            debug!("Checking resolution for connector {:?}-{:?}", connector_info.interface(), connector_info.interface_id());
            if let Some(encoder_id) = connector_info.current_encoder() {
                debug!("Fetching encoder info for ID {:?}", encoder_id);
                let encoder_info = card.get_encoder(encoder_id)?;
                if let Some(crtc_id) = encoder_info.crtc() {
                    debug!("Fetching CRTC info for ID {:?}", crtc_id);
                    let crtc_info = card.get_crtc(crtc_id)?;
                    if let Some(mode) = crtc_info.mode() {
                        let res_str = format!("{}x{}", mode.size().0, mode.size().1);
                        debug!("Current resolution found: {}", res_str);
                        current_resolution_string.push_str(&res_str);
                    }
                }
            }
        }
        Ok(())
    })?;

    if current_resolution_string.is_empty() {
        warn!("No current resolution found for screen: {:?}", screen);
        current_resolution_string.push_str("No current resolution found.");
    } else {
        info!("Current resolution retrieved: {}", current_resolution_string);
    }
    Ok(current_resolution_string)
}

/// Retrieves the current refresh rate for a specified screen.
///
/// # Arguments
/// - `screen`: Optional screen name to filter the refresh rate.
///
/// # Returns
/// - A string in the format "REFRESH_RATEHz" (e.g., "60Hz").
pub fn drm_current_refresh(screen: Option<&str>) -> Result<String, Box<dyn Error>> {
    info!("Getting current refresh rate for screen: {:?}", screen);
    debug!("Opening first available DRM card");
    let card = Card::open_first_available()?;
    let mut current_refresh_string = String::new();

    debug!("Iterating over connectors to find current refresh rate");
    for_each_connector(&card, screen, |connector_info| -> Result<(), Box<dyn Error>> {
        if connector_info.state() == connector::State::Connected {
            debug!("Checking refresh rate for connector {:?}-{:?}", connector_info.interface(), connector_info.interface_id());
            if let Some(encoder_id) = connector_info.current_encoder() {
                debug!("Fetching encoder info for ID {:?}", encoder_id);
                let encoder_info = card.get_encoder(encoder_id)?;
                if let Some(crtc_id) = encoder_info.crtc() {
                    debug!("Fetching CRTC info for ID {:?}", crtc_id);
                    let crtc_info = card.get_crtc(crtc_id)?;
                    if let Some(mode) = crtc_info.mode() {
                        let refresh_str = format!("{}Hz", mode.vrefresh());
                        debug!("Current refresh rate found: {}", refresh_str);
                        current_refresh_string.push_str(&refresh_str);
                    }
                }
            }
        }
        Ok(())
    })?;

    if current_refresh_string.is_empty() {
        warn!("No current refresh rate found for screen: {:?}", screen);
        current_refresh_string.push_str("No current refresh rate found.");
    } else {
        info!("Current refresh rate retrieved: {}", current_refresh_string);
    }
    Ok(current_refresh_string)
}

/// Sets a specific display mode (resolution and refresh rate) for a screen.
///
/// # Arguments
/// - `screen`: Optional screen name to apply the mode to.
/// - `width`: Desired width in pixels.
/// - `height`: Desired height in pixels.
/// - `vrefresh`: Desired refresh rate in Hz.
///
/// # Returns
/// - `Ok(())` if the mode is set successfully.
/// - `Err` if no matching mode is found.
pub fn drm_set_mode(screen: Option<&str>, width: i32, height: i32, vrefresh: i32) -> Result<(), Box<dyn Error>> {
    info!("Setting mode {}x{}@{}Hz for screen: {:?}", width, height, vrefresh, screen);
    debug!("Opening first available DRM card");
    let card = Card::open_first_available()?;
    let mut mode_set = false;

    debug!("Iterating over connectors to set display mode");
    for_each_connector(&card, screen, |connector_info| -> Result<(), Box<dyn Error>> {
        if connector_info.state() == connector::State::Connected {
            let interface_str = format!("{:?}", connector_info.interface());
            if let Some(screen_name) = screen {
                if !interface_str.contains(screen_name) {
                    debug!("Skipping connector {} - doesn't match screen {}", interface_str, screen_name);
                    return Ok(());
                }
            }
            debug!("Processing connected connector: {}", interface_str);

            if let Some(encoder_id) = connector_info.current_encoder() {
                debug!("Fetching encoder info for ID {:?}", encoder_id);
                let encoder_info = card.get_encoder(encoder_id)?;
                if let Some(crtc_id) = encoder_info.crtc() {
                    debug!("Fetching CRTC info for ID {:?}", crtc_id);
                    let crtc_info = card.get_crtc(crtc_id)?;
                    for mode in connector_info.modes() {
                        if mode.size().0 == width as u16 &&
                           mode.size().1 == height as u16 &&
                           mode.vrefresh() == vrefresh as u32 {
                            debug!("Matching mode found: {}x{}@{}Hz", width, height, vrefresh);
                            card.set_crtc(crtc_id, crtc_info.framebuffer(), (0, 0), &[connector_info.handle()], Some(*mode))?;
                            info!("Display mode set to {}x{}@{}Hz on {}", width, height, vrefresh, interface_str);
                            mode_set = true;
                            return Ok(());
                        }
                    }
                    debug!("No matching mode found for {}x{}@{}Hz on {}", width, height, vrefresh, interface_str);
                }
            }
        }
        Ok(())
    })?;

    if !mode_set {
        error!("No matching mode found for {}x{}@{}Hz on screen {:?}", width, height, vrefresh, screen);
        return Err("No matching mode found for specified screen".into());
    }
    info!("Mode setting completed successfully");
    Ok(())
}

/// Sets the active output to a specified connector.
///
/// # Arguments
/// - `output`: The name of the output to activate (e.g., "HDMI-A-1").
///
/// # Returns
/// - `Ok(())` if the output is set successfully.
/// - `Err` if the specified output is not found.
pub fn drm_set_output(output: &str) -> Result<(), Box<dyn Error>> {
    info!("Setting output to {}", output);
    debug!("Opening first available DRM card");
    let card = Card::open_first_available()?;
    let mut found = false;

    debug!("Iterating over connectors to set output");
    for_each_connector(&card, None, |connector_info| -> Result<(), Box<dyn Error>> {
        let interface_str = format!("{:?}", connector_info.interface());
        if interface_str.contains(output) {
            debug!("Found matching output: {}", interface_str);
            if let Some(encoder_id) = connector_info.current_encoder() {
                debug!("Fetching encoder info for ID {:?}", encoder_id);
                let encoder_info = card.get_encoder(encoder_id)?;
                if let Some(crtc_id) = encoder_info.crtc() {
                    debug!("Fetching CRTC info for ID {:?}", crtc_id);
                    let crtc_info = card.get_crtc(crtc_id)?;
                    if let Some(mode) = crtc_info.mode() {
                        debug!("Setting CRTC with current mode for output {}", interface_str);
                        card.set_crtc(crtc_id, crtc_info.framebuffer(), (0, 0), &[connector_info.handle()], Some(mode))?;
                        info!("Output set to {}", output);
                        found = true;
                    }
                }
            }
        }
        Ok(())
    })?;

    if !found {
        error!("Output {} not found", output);
        return Err(format!("Output {} not found", output).into());
    }
    info!("Output setting completed successfully");
    Ok(())
}

/// Sets the rotation of a specified screen.
///
/// # Arguments
/// - `screen`: Optional screen name to apply the rotation to (e.g., "HDMI", "DP").
/// - `rotation`: The rotation value ("0", "90", "180", "270").
///
/// # Returns
/// - `Ok(())` if the rotation is set successfully.
/// - `Err` if the rotation value is invalid or no matching screen is found.
pub fn drm_set_rotation(screen: Option<&str>, rotation: &str) -> Result<(), Box<dyn Error>> {
    info!("Starting rotation setting process for rotation '{}' on screen {:?}", rotation, screen);
    debug!("Opening first available DRM card");
    let card = Card::open_first_available()?;

    debug!("Parsing rotation input: '{}'", rotation);
    let rotation_value = match rotation.to_lowercase().as_str() {
        "0" => 0,
        "90" => 1,
        "180" => 2,
        "270" => 3,
        _ => {
            error!("Invalid rotation value provided: '{}'. Expected '0', '90', '180', '270'", rotation);
            return Err("Invalid rotation value (use 0, 90, 180, 270)".into());
        }
    };
    debug!("Rotation value mapped to integer: {}", rotation_value);

    let mut rotation_set = false;
    debug!("Beginning connector iteration to apply rotation");
    for_each_connector(&card, screen, |connector_info| -> Result<(), Box<dyn Error>> {
        if connector_info.state() == connector::State::Connected {
            let interface_str = format!("{:?}", connector_info.interface());
            if let Some(screen_name) = screen {
                if !interface_str.contains(screen_name) {
                    debug!("Skipping connector '{}' - does not match screen filter '{}'", interface_str, screen_name);
                    return Ok(());
                }
                debug!("Connector '{}' matches screen filter '{}'", interface_str, screen_name);
            } else {
                debug!("No screen filter provided; processing connector '{}'", interface_str);
            }

            debug!("Fetching properties for connector handle {:?}", connector_info.handle());
            let properties = card.get_properties(connector_info.handle())?;

            for (prop_id, _) in properties {
                let prop_info = card.get_property(prop_id)?;
                let prop_name = prop_info.name().to_string_lossy();
                if prop_name.contains("rotation") {
                    debug!("Applying rotation property ID {:?} with value {} to connector '{}'", prop_id, rotation_value, interface_str);
                    card.set_property(connector_info.handle(), prop_id, rotation_value as u64)?;
                    info!("Successfully set rotation to '{}' on connector '{}'", rotation, interface_str);
                    rotation_set = true;
                    return Ok(());
                }
                debug!("Property '{}' (ID {:?}) does not match 'rotation'", prop_name, prop_id);
            }
            warn!("No 'rotation' property found for connector '{}'", interface_str);
        } else {
            debug!("Skipping disconnected connector {:?}", connector_info.interface());
        }
        Ok(())
    })?;

    if !rotation_set {
        error!("Failed to set rotation '{}': No matching connected screen found for {:?}", rotation, screen);
        return Err("No matching connected screen found for rotation".into());
    }

    info!("Rotation setting process completed successfully for '{}'", rotation);
    Ok(())
}

/// Attempts to capture a screenshot from the DRM device.
///
/// # Returns
/// - `Ok(())` if the attempt completes (though capture is not fully implemented).
/// - `Err` if an error occurs during connector iteration.
///
/// # Note
/// This function is incomplete and requires framebuffer reading to fully implement screenshot capture.
pub fn drm_get_screenshot() -> Result<(), Box<dyn Error>> {
    info!("Attempting to capture screenshot");
    debug!("Opening first available DRM card");
    let card = Card::open_first_available()?;

    debug!("Iterating over connectors to capture screenshot");
    for_each_connector(&card, None, |connector_info| -> Result<(), Box<dyn Error>> {
        if connector_info.state() == connector::State::Connected {
            debug!("Processing connected connector {:?}", connector_info.interface());
            if let Some(encoder_id) = connector_info.current_encoder() {
                debug!("Fetching encoder info for ID {:?}", encoder_id);
                let encoder_info = card.get_encoder(encoder_id)?;
                if let Some(crtc_id) = encoder_info.crtc() {
                    debug!("Fetching CRTC info for ID {:?}", crtc_id);
                    let crtc_info = card.get_crtc(crtc_id)?;
                    if let Some(fb_id) = crtc_info.framebuffer() {
                        debug!("Fetching framebuffer info for ID {:?}", fb_id);
                        let _fb_info = card.get_framebuffer(fb_id)?;
                        warn!("Screenshot capture not fully implemented - framebuffer reading needed");
                    }
                }
            }
        }
        Ok(())
    })?;
    info!("Screenshot capture attempt completed");
    Ok(())
}

/// Sets the display to its maximum supported resolution for a specified screen.
///
/// # Arguments
/// - `screen`: Optional screen name to apply the maximum resolution to.
/// - `_max_resolution`: Currently unused optional parameter for a specific max resolution.
///
/// # Returns
/// - `Ok(())` if the maximum resolution is set successfully.
/// - `Err` if no suitable resolution is found.
pub fn drm_to_max_resolution(screen: Option<&str>, _max_resolution: Option<String>) -> Result<(), Box<dyn Error>> {
    info!("Setting maximum resolution for screen: {:?}", screen);
    debug!("Opening first available DRM card");
    let card = Card::open_first_available()?;
    let mut max_width = 0;
    let mut max_height = 0;
    let mut best_mode = None;
    let mut target_connector = None;

    // Find the highest resolution mode available
    debug!("Iterating over connectors to find maximum resolution");
    for_each_connector(&card, screen, |connector_info| -> Result<(), Box<dyn Error>> {
        if connector_info.state() == connector::State::Connected {
            let interface_str = format!("{:?}", connector_info.interface());
            if let Some(screen_name) = screen {
                if !interface_str.contains(screen_name) {
                    debug!("Skipping connector {} - doesn't match screen {}", interface_str, screen_name);
                    return Ok(());
                }
            }
            debug!("Processing connected connector: {}", interface_str);

            for mode in connector_info.modes() {
                if mode.size().0 > max_width || (mode.size().0 == max_width && mode.size().1 > max_height) {
                    max_width = mode.size().0;
                    max_height = mode.size().1;
                    best_mode = Some(*mode);
                    target_connector = Some(connector_info.handle());
                    debug!("Found better resolution: {}x{}", max_width, max_height);
                }
            }
        }
        Ok(())
    })?;

    // Apply the best mode found
    if let (Some(mode), Some(connector_handle)) = (best_mode, target_connector) {
        debug!("Best mode found: {}x{}", max_width, max_height);
        debug!("Iterating over connectors to apply maximum resolution");
        for_each_connector(&card, screen, |connector_info| -> Result<(), Box<dyn Error>> {
            if connector_info.handle() == connector_handle {
                if let Some(encoder_id) = connector_info.current_encoder() {
                    debug!("Fetching encoder info for ID {:?}", encoder_id);
                    let encoder_info = card.get_encoder(encoder_id)?;
                    if let Some(crtc_id) = encoder_info.crtc() {
                        debug!("Fetching CRTC info for ID {:?}", crtc_id);
                        let crtc_info = card.get_crtc(crtc_id)?;
                        debug!("Setting CRTC to max resolution {}x{}", max_width, max_height);
                        card.set_crtc(crtc_id, crtc_info.framebuffer(), (0, 0), &[connector_info.handle()], Some(mode))?;
                        info!("Set to max resolution {}x{} on {:?}", max_width, max_height, connector_info.interface());
                    }
                }
            }
            Ok(())
        })?;
    } else {
        error!("No suitable resolution found for screen {:?}", screen);
        return Err("No suitable resolution found for specified screen".into());
    }
    info!("Maximum resolution setting completed successfully");
    Ok(())
}
