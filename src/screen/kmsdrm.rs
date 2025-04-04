use drm::Device;
use drm::control::{Device as ControlDevice, connector};
use log::{debug, error, info, warn};
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::os::unix::io::{AsFd, BorrowedFd};
use std::path::Path;

/// Represents a DRM (Direct Rendering Manager) device card.
///
/// This struct wraps a `File` object representing an opened DRM device file (e.g., `/dev/dri/card0`).
/// It provides a simple abstraction for interacting with DRM devices.
#[derive(Debug)]
struct Card(File);

/// Implements the `AsFd` trait to allow `Card` to provide a file descriptor.
///
/// This is necessary for low-level operations that require access to the file descriptor.
impl AsFd for Card {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

/// Implements the `Device` trait for `Card`, enabling basic DRM operations.
///
/// This trait provides the foundational DRM functionality, such as resource management.
impl Device for Card {}

/// Implements the `ControlDevice` trait for `Card`, enabling control-specific DRM operations.
///
/// This trait allows for advanced control over connectors, CRTCs, and other DRM components.
impl ControlDevice for Card {}

impl Card {
    /// Opens the available DRM device in `/dev/dri/`.
    ///
    /// Iterates through the directory entries and attempts to open a device starting with "card".
    /// If multiple devices are found, the one with the most available connectors is chosen.
    ///
    /// # Returns
    /// - `Ok(Card)` if a device is successfully opened.
    /// - `Err(Box<dyn Error>)` if no functional DRM device is found.
    fn open_available() -> Result<Self, Box<dyn Error>> {
        let dri_path = Path::new("/dev/dri/");
        info!("Searching for DRM devices in {:?}", dri_path);

        let mut last_error = None;
        let mut best_card = None;
        let mut max_connectors = 0;

        for entry in dri_path.read_dir()?.filter_map(Result::ok) {
            let path = entry.path();
            if let Some(file_name) = path.file_name().map(|f| f.to_string_lossy()) {
                if file_name.starts_with("card") {
                    debug!("Attempting to open device: {:?}", path);
                    let mut options = OpenOptions::new();
                    options.read(true).write(true);

                    match options.open(&path) {
                        Ok(file) => {
                            let card = Card(file);
                            match card.resource_handles() {
                                Ok(resources) => {
                                    let num_connectors = resources.connectors().len();
                                    info!("Device {:?} opened with {} connectors.", path, num_connectors);

                                    if num_connectors > max_connectors {
                                        best_card = Some(card);
                                        max_connectors = num_connectors;
                                    }
                                }
                                Err(e) if e.raw_os_error() == Some(95) => {
                                    debug!("Device {:?} does not support basic operations (Error 95), ignoring...", path);
                                    last_error = Some(e);
                                }
                                Err(e) => {
                                    error!("Error obtaining resources from device {:?}: {:?}", path, e);
                                    last_error = Some(e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to open device {:?}: {:?}", path, e);
                            last_error = Some(e);
                        }
                    }
                }
            }
        }

        if let Some(card) = best_card {
            Ok(card)
        } else {
            error!("No functional DRM device found in {:?}.", dri_path);
            Err(last_error.map_or_else(
                || Box::<dyn Error>::from("No DRM device found or accessible"),
                |e| Box::new(e),
            ))
        }
    }
}


/// Iterates over all available connectors of the DRM device, applying a given function.
///
/// This function retrieves connector information and filters based on an optional screen name.
///
/// # Arguments
/// - `drm_device`: A reference to the `Card` representing the DRM device.
/// - `screen`: An optional screen name to filter connectors (e.g., "HDMI", "DP").
/// - `f`: A closure that is executed for each valid connector found.
///
/// # Returns
/// - `Ok(())` if iteration completes successfully.
/// - `Err(Box<dyn Error>)` if an error occurs during connector processing.
fn for_each_connector<F>(
    drm_device: &Card,
    screen: Option<&str>,
    mut f: F,
) -> Result<(), Box<dyn Error>>
where
    F: FnMut(&connector::Info) -> Result<(), Box<dyn Error>>,
{
    debug!("Fetching resource handles for DRM device");

    // Retrieve all connectors once to avoid repeated queries
    let resources = drm_device.resource_handles()?;
    let connectors = resources.connectors();

    debug!("Found {} connectors", connectors.len());

    connectors
        .iter()
        .filter_map(|&connector_handle| {
            // Fetch connector info; skip if failed
            match drm_device.get_connector(connector_handle, true) {
                Ok(connector_info) => Some(connector_info),
                Err(e) => {
                    warn!(
                        "Failed to get info for connector {:?}: {}",
                        connector_handle, e
                    );
                    None
                }
            }
        })
        .filter(|connector_info| {
            // Apply screen filter if provided
            match screen {
                Some(screen_name) if format!("{:?}", connector_info.interface()) != screen_name => {
                    debug!(
                        "Skipping connector {:?} - doesn't match screen {}",
                        connector_info.interface(),
                        screen_name
                    );
                    false
                }
                _ => true, // No filter means process all connectors
            }
        })
        .try_for_each(|connector_info| f(&connector_info))?; // Execute the provided function

    debug!("Connector iteration completed successfully");
    Ok(())
}

/// Lists all available display modes for a specified screen.
///
/// Includes both hardcoded maximum resolutions and connector-specific modes.
///
/// # Arguments
/// - `screen`: Optional screen name to filter modes (e.g., "HDMI", "DP").
///
/// # Returns
/// - `Ok(String)` containing all available modes in a formatted string.
/// - `Err(Box<dyn Error>)` if an error occurs while accessing the DRM device.
///
/// # Examples
/// ```
/// let modes = drm_list_modes(Some("HDMI")).unwrap();
/// println!("{}", modes);
/// ```
pub fn drm_list_modes(screen: Option<&str>) -> Result<String, Box<dyn Error>> {
    debug!("Opening first available DRM card");
    let card = Card::open_available()?;
    let mut modes_string = String::new();
    let mut total_modes = 0;

    // Primeiro passamos para contar o total de modos
    for_each_connector(&card, screen, |connector_info| {
        total_modes += connector_info.modes().len();
        Ok(())
    })?;

    let mut modes_processed = 0;
    debug!("Iterating over connectors to list modes");
    for_each_connector(&card, screen, |connector_info| {
        debug!(
            "Processing connector {:?}",
            connector_info.interface()
        );
        let modes = connector_info.modes();
        debug!("Found {} modes for connector", modes.len());

        for mode in modes {
            modes_processed += 1;
            let mode_str = format!(
                "{}x{}@{}:{:?} {}x{}@{}Hz",
                mode.size().0,
                mode.size().1,
                mode.vrefresh(),
                connector_info.interface(),
                mode.size().0,
                mode.size().1,
                mode.vrefresh()
            );
            debug!("Adding mode: {}", mode_str.trim());
            modes_string.push_str(&mode_str);

            // Adiciona nova linha apenas se não for o último modo
            if modes_processed < total_modes {
                modes_string.push('\n');
            }
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
/// - `Ok(String)` listing all connector names (e.g., "HDMI-A-1", "DP-1").
/// - `Err(Box<dyn Error>)` if an error occurs while accessing the DRM device.
///
/// # Examples
/// ```
/// let outputs = drm_list_outputs().unwrap();
/// println!("{}", outputs);
/// ```
pub fn drm_list_outputs() -> Result<String, Box<dyn Error>> {
    debug!("Opening first available DRM card");
    let card = Card::open_available()?;
    let mut outputs_string = String::new();
    let mut first = true; // Used to avoid leading newline

    debug!("Iterating over connectors to list outputs");
    for_each_connector(&card, None, |connector_info| {
        let output_str = format!(
            "{:?}", connector_info.interface()
        );
        debug!("Found output: {}", output_str);

        if !first {
            outputs_string.push_str("\n");
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
/// - `screen`: Optional screen name to filter the mode (e.g., "HDMI", "DP").
///
/// # Returns
/// - `Ok(String)` in the format "WIDTHxHEIGHT@REFRESH_RATE" (e.g., "1920x1080@60Hz").
/// - `Err(Box<dyn Error>)` if an error occurs or no mode is found.
///
/// # Examples
/// ```
/// let mode = drm_current_mode(Some("HDMI")).unwrap();
/// println!("Current mode: {}", mode);
/// ```
pub fn drm_current_mode(screen: Option<&str>) -> Result<String, Box<dyn Error>> {
    debug!("Opening first available DRM card");
    let card = Card::open_available()?;
    let mut current_mode_string = String::new();

    debug!("Iterating over connectors to find current mode");
    for_each_connector(
        &card,
        screen,
        |connector_info| -> Result<(), Box<dyn Error>> {
            if connector_info.state() == connector::State::Connected {
                debug!(
                    "Checking connected connector {:?}",
                    connector_info.interface()
                );
                if let Some(encoder_id) = connector_info.current_encoder() {
                    debug!("Fetching encoder info for ID {:?}", encoder_id);
                    let encoder_info = card.get_encoder(encoder_id)?;
                    if let Some(crtc_id) = encoder_info.crtc() {
                        debug!("Fetching CRTC info for ID {:?}", crtc_id);
                        let crtc_info = card.get_crtc(crtc_id)?;
                        if let Some(mode) = crtc_info.mode() {
                            let mode_str = format!(
                                "{}x{}@{}",
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
        },
    )?;

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
/// - `Ok(String)` with the name of the connected output (e.g., "HDMI-A-1").
/// - `Err(Box<dyn Error>)` if an error occurs or no connected output is found.
///
/// # Examples
/// ```
/// let output = drm_current_output().unwrap();
/// println!("Current output: {}", output);
/// ```
pub fn drm_current_output() -> Result<String, Box<dyn Error>> {
    debug!("Opening first available DRM card");
    let card = Card::open_available()?;
    let mut current_output_string = String::new();

    debug!("Iterating over connectors to find current output");
    for_each_connector(
        &card,
        None,
        |connector_info| -> Result<(), Box<dyn Error>> {
            if connector_info.state() == connector::State::Connected {
                let output_str = format!(
                    "{:?}", connector_info.interface()
                );
                debug!("Found connected output: {}", output_str);
                current_output_string.push_str(&output_str);
            }
            Ok(())
        },
    )?;

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
/// - `screen`: Optional screen name to filter the resolution (e.g., "HDMI", "DP").
///
/// # Returns
/// - `Ok(String)` in the format "WIDTHxHEIGHT" (e.g., "1920x1080").
/// - `Err(Box<dyn Error>)` if an error occurs or no resolution is found.
///
/// # Examples
/// ```
/// let resolution = drm_current_resolution(Some("HDMI")).unwrap();
/// println!("Current resolution: {}", resolution);
/// ```
pub fn drm_current_resolution(screen: Option<&str>) -> Result<String, Box<dyn Error>> {
    debug!("Opening first available DRM card");
    let card = Card::open_available()?;
    let mut current_resolution_string = String::new();

    debug!("Iterating over connectors to find current resolution");
    for_each_connector(
        &card,
        screen,
        |connector_info| -> Result<(), Box<dyn Error>> {
            if connector_info.state() == connector::State::Connected {
                debug!(
                    "Checking resolution for connector {:?}",
                    connector_info.interface()
                );
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
        },
    )?;

    if current_resolution_string.is_empty() {
        warn!("No current resolution found for screen: {:?}", screen);
        current_resolution_string.push_str("No current resolution found.");
    } else {
        info!(
            "Current resolution retrieved: {}",
            current_resolution_string
        );
    }
    Ok(current_resolution_string)
}

/// Retrieves the current refresh rate for a specified screen.
///
/// # Arguments
/// - `screen`: Optional screen name to filter the refresh rate (e.g., "HDMI", "DP").
///
/// # Returns
/// - `Ok(String)` in the format "REFRESH_RATEHz" (e.g., "60Hz").
/// - `Err(Box<dyn Error>)` if an error occurs or no refresh rate is found.
///
/// # Examples
/// ```
/// let refresh = drm_current_refresh(Some("HDMI")).unwrap();
/// println!("Current refresh rate: {}", refresh);
/// ```
pub fn drm_current_refresh(screen: Option<&str>) -> Result<String, Box<dyn Error>> {
    debug!("Opening first available DRM card");
    let card = Card::open_available()?;
    let mut current_refresh_string = String::new();

    debug!("Iterating over connectors to find current refresh rate");
    for_each_connector(
        &card,
        screen,
        |connector_info| -> Result<(), Box<dyn Error>> {
            if connector_info.state() == connector::State::Connected {
                debug!(
                    "Checking refresh rate for connector {:?}",
                    connector_info.interface()
                );
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
        },
    )?;

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
/// Applies the mode to the first matching connected connector.
///
/// # Arguments
/// - `screen`: Optional screen name to apply the mode to (e.g., "HDMI", "DP").
/// - `width`: Desired width in pixels.
/// - `height`: Desired height in pixels.
/// - `vrefresh`: Desired refresh rate in Hz.
///
/// # Returns
/// - `Ok(())` if the mode is set successfully.
/// - `Err(Box<dyn Error>)` if no matching mode is found or an error occurs.
///
/// # Examples
/// ```
/// drm_set_mode(Some("HDMI"), 1920, 1080, 60).unwrap();
/// ```
pub fn drm_set_mode(
    screen: Option<&str>,
    width: i32,
    height: i32,
    vrefresh: i32,
) -> Result<(), Box<dyn Error>> {
    info!(
        "Setting mode {}x{}@{}Hz for screen: {:?}",
        width, height, vrefresh, screen
    );
    debug!("Opening first available DRM card");
    let card = Card::open_available()?;

    debug!("Iterating over connectors to set display mode");
    for_each_connector(
        &card,
        screen,
        |connector_info| -> Result<(), Box<dyn Error>> {
            if connector_info.state() != connector::State::Connected {
                return Ok(());
            }

            let interface = format!("{:?}", connector_info.interface());
            if let Some(screen_name) = screen {
                if interface != screen_name {
                    debug!("Skipping connector {} - doesn't match screen {}", interface, screen_name);
                    return Ok(());
                }
            }

            debug!("Processing connected connector: {}", interface);

            let encoder_id = match connector_info.current_encoder() {
                Some(id) => id,
                None => {
                    warn!("No encoder found for connector {}", interface);
                    return Ok(());
                }
            };

            let encoder_info = card.get_encoder(encoder_id)?;
            let crtc_id = match encoder_info.crtc() {
                Some(id) => id,
                None => {
                    warn!("No CRTC found for encoder {:?} on {}", encoder_id, interface);
                    return Ok(());
                }
            };

            let crtc_info = card.get_crtc(crtc_id)?;
            let current_fb = crtc_info.framebuffer();

            let target_mode = connector_info.modes().iter().find(|mode| {
                mode.size().0 == width as u16 &&
                mode.size().1 == height as u16 &&
                mode.vrefresh() == vrefresh as u32
            }).ok_or("Mode not found")?;

            card.set_crtc(
                crtc_id,
                current_fb,
                (0, 0),
                &[connector_info.handle()],
                Some(*target_mode),
            )?;

            debug!("No matching mode found for {}x{}@{}Hz on {}", width, height, vrefresh, interface);
            Ok(())
        },
    )?;

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
/// - `Err(Box<dyn Error>)` if the specified output is not found or an error occurs.
///
/// # Examples
/// ```
/// drm_set_output("HDMI-A-1").unwrap();
/// ```
pub fn drm_set_output(output: &str) -> Result<(), Box<dyn Error>> {
    debug!("Opening first available DRM card");
    let card = Card::open_available()?;
    let mut found = false;

    debug!("Iterating over connectors to set output");
    for_each_connector(
        &card,
        None,
        |connector_info| -> Result<(), Box<dyn Error>> {
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
                            debug!(
                                "Setting CRTC with current mode for output {}",
                                interface_str
                            );
                            card.set_crtc(
                                crtc_id,
                                crtc_info.framebuffer(),
                                (0, 0),
                                &[connector_info.handle()],
                                Some(mode),
                            )?;
                            info!("Output set to {}", output);
                            found = true;
                        }
                    }
                }
            }
            Ok(())
        },
    )?;

    if !found {
        error!("Output {} not found", output);
        return Err(format!("Output {} not found", output).into());
    }
    info!("Output setting completed successfully");
    Ok(())
}

/// Sets the rotation of a specified screen.
///
/// Applies the rotation to the first matching connected connector with a "rotation" property.
///
/// # Arguments
/// - `screen`: Optional screen name to apply the rotation to (e.g., "HDMI", "DP").
/// - `rotation`: The rotation value as a string ("0", "90", "180", "270").
///
/// # Returns
/// - `Ok(())` if the rotation is setNor successfully.
/// - `Err(Box<dyn Error>)` if the rotation value is invalid or no matching screen is found.
///
/// # Examples
/// ```
/// drm_set_rotation(Some("HDMI"), "90").unwrap();
/// ```
pub fn drm_set_rotation(screen: Option<&str>, rotation: &str) -> Result<(), Box<dyn Error>> {
    debug!("Opening first available DRM card");
    let card = Card::open_available()?;

    // Map rotation string to integer value
    debug!("Parsing rotation input: '{}'", rotation);
    let rotation_value = match rotation.to_lowercase().as_str() {
        "0" => 0,
        "90" => 1,
        "180" => 2,
        "270" => 3,
        _ => {
            error!(
                "Invalid rotation value provided: '{}'. Expected '0', '90', '180', '270'",
                rotation
            );
            return Err("Invalid rotation value (use 0, 90, 180, 270)".into());
        }
    };
    debug!("Rotation value mapped to integer: {}", rotation_value);

    let mut rotation_set = false;
    debug!("Beginning connector iteration to apply rotation");
    for_each_connector(
        &card,
        screen,
        |connector_info| -> Result<(), Box<dyn Error>> {
            if connector_info.state() == connector::State::Connected {
                let interface_str = format!("{:?}", connector_info.interface());
                if let Some(screen_name) = screen {
                    if !interface_str.contains(screen_name) {
                        debug!(
                            "Skipping connector '{}' - does not match screen filter '{}'",
                            interface_str, screen_name
                        );
                        return Ok(());
                    }
                    debug!(
                        "Connector '{}' matches screen filter '{}'",
                        interface_str, screen_name
                    );
                } else {
                    debug!(
                        "No screen filter provided; processing connector '{}'",
                        interface_str
                    );
                }

                // Check properties for "rotation"
                debug!(
                    "Fetching properties for connector handle {:?}",
                    connector_info.handle()
                );
                let properties = card.get_properties(connector_info.handle())?;

                for (prop_id, _) in properties {
                    let prop_info = card.get_property(prop_id)?;
                    let prop_name = prop_info.name().to_string_lossy();
                    if prop_name.contains("rotation") {
                        debug!(
                            "Applying rotation property ID {:?} with value {} to connector '{}'",
                            prop_id, rotation_value, interface_str
                        );
                        card.set_property(connector_info.handle(), prop_id, rotation_value as u64)?;
                        info!(
                            "Successfully set rotation to '{}' on connector '{}'",
                            rotation, interface_str
                        );
                        rotation_set = true;
                        return Ok(());
                    }
                    debug!(
                        "Property '{}' (ID {:?}) does not match 'rotation'",
                        prop_name, prop_id
                    );
                }
                warn!(
                    "No 'rotation' property found for connector '{}'",
                    interface_str
                );
            } else {
                debug!(
                    "Skipping disconnected connector {:?}",
                    connector_info.interface()
                );
            }
            Ok(())
        },
    )?;

    if !rotation_set {
        error!(
            "Failed to set rotation '{}': No matching connected screen found for {:?}",
            rotation, screen
        );
        return Err("No matching connected screen found for rotation".into());
    }

    info!(
        "Rotation setting process completed successfully for '{}'",
        rotation
    );
    Ok(())
}

/// Retrieves a screenshot from the DRM device.
///
/// # Returns
/// - `Result<(), Box<dyn Error>>` (currently unimplemented).
///
/// # Notes
/// - This function is a placeholder and needs implementation.
pub fn drm_get_screenshot() -> Result<(), Box<dyn Error>> {
    Err("TODO: Needs implementation.".into())
}

/// Sets the display to its maximum supported resolution for a specified screen.
///
/// Automatically detects and applies the highest resolution available.
///
/// # Arguments
/// - `screen`: Optional screen name to apply the maximum resolution to (e.g., "HDMI", "DP").
/// - `_max_resolution`: Currently unused optional parameter for a specific max resolution.
///
/// # Returns
/// - `Ok(())` if the maximum resolution is set successfully.
/// - `Err(Box<dyn Error>)` if no suitable resolution is found or an error occurs.
///
/// # Examples
/// ```
/// drm_to_max_resolution(Some("HDMI"), None).unwrap();
/// ```
pub fn drm_to_max_resolution(
    screen: Option<&str>,
    _max_resolution: Option<String>,
) -> Result<(), Box<dyn Error>> {
    debug!("Opening first available DRM card");
    let card = Card::open_available()?;
    let mut max_width = 0;
    let mut max_height = 0;
    let mut best_mode = None;
    let mut target_connector = None;

    // First pass: find the highest resolution mode
    debug!("Iterating over connectors to find maximum resolution");
    for_each_connector(
        &card,
        screen,
        |connector_info| -> Result<(), Box<dyn Error>> {
            if connector_info.state() == connector::State::Connected {
                let interface_str = format!("{:?}", connector_info.interface());
                if let Some(screen_name) = screen {
                    if !interface_str.contains(screen_name) {
                        debug!(
                            "Skipping connector {} - doesn't match screen {}",
                            interface_str, screen_name
                        );
                        return Ok(());
                    }
                }
                debug!("Processing connected connector: {}", interface_str);

                for mode in connector_info.modes() {
                    if mode.size().0 > max_width
                        || (mode.size().0 == max_width && mode.size().1 > max_height)
                    {
                        max_width = mode.size().0;
                        max_height = mode.size().1;
                        best_mode = Some(*mode);
                        target_connector = Some(connector_info.handle());
                        debug!("Found better resolution: {}x{}", max_width, max_height);
                    }
                }
            }
            Ok(())
        },
    )?;

    // Second pass: apply the best mode found
    if let (Some(mode), Some(connector_handle)) = (best_mode, target_connector) {
        debug!("Best mode found: {}x{}", max_width, max_height);
        debug!("Iterating over connectors to apply maximum resolution");
        for_each_connector(
            &card,
            screen,
            |connector_info| -> Result<(), Box<dyn Error>> {
                if connector_info.handle() == connector_handle {
                    if let Some(encoder_id) = connector_info.current_encoder() {
                        debug!("Fetching encoder info for ID {:?}", encoder_id);
                        let encoder_info = card.get_encoder(encoder_id)?;
                        if let Some(crtc_id) = encoder_info.crtc() {
                            debug!("Fetching CRTC info for ID {:?}", crtc_id);
                            let crtc_info = card.get_crtc(crtc_id)?;
                            debug!(
                                "Setting CRTC to max resolution {}x{}",
                                max_width, max_height
                            );
                            card.set_crtc(
                                crtc_id,
                                crtc_info.framebuffer(),
                                (0, 0),
                                &[connector_info.handle()],
                                Some(mode),
                            )?;
                            info!(
                                "Set to max resolution {}x{} on {:?}",
                                max_width,
                                max_height,
                                connector_info.interface()
                            );
                        }
                    }
                }
                Ok(())
            },
        )?;
    } else {
        error!("No suitable resolution found for screen {:?}", screen);
        return Err("No suitable resolution found for specified screen".into());
    }
    info!("Maximum resolution setting completed successfully");
    Ok(())
}
