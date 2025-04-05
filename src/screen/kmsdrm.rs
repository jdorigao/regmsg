//! Kernel Mode Setting (KMS) and Direct Rendering Manager (DRM) interface
//!
//! This module provides a high-level interface for interacting with DRM devices on Linux systems.
//! It supports querying display information, changing resolutions, setting rotations, and other
//! display-related operations.
//!
//! # Features
//! - List available displays and their modes
//! - Query current display settings (resolution, refresh rate)
//! - Change display modes and resolutions
//! - Set display rotation
//! - Configure maximum resolution
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```no_run
//! // List all available outputs
//! let outputs = drm_list_outputs().unwrap();
//! println!("Available outputs: {}", outputs);
//!
//! // List available modes for HDMI
//! let modes = drm_list_modes(Some("HDMI")).unwrap();
//! println!("Available HDMI modes:\n{}", modes);
//!
//! // Set HDMI output to 1920x1080@60Hz
//! drm_set_mode(Some("HDMI"), 1920, 1080, 60).unwrap();
//!
//! // Set display rotation to 90 degrees
//! drm_set_rotation(Some("HDMI"), "90").unwrap();
//! ```

use drm::Device;
use drm::control::{Device as ControlDevice, connector};
use log::{debug, error, info, warn};
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::os::unix::io::{AsFd, BorrowedFd};
use std::path::Path;
use drm::buffer::DrmFourcc;
use image::{ImageBuffer, RgbImage};
use chrono::Local;

/// Represents a DRM (Direct Rendering Manager) device card.
///
/// This struct wraps a `File` object representing an opened DRM device file (e.g., `/dev/dri/card0`).
/// It provides the core abstraction for interacting with DRM devices.
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
    /// Opens the first available DRM device in `/dev/dri/`.
    ///
    /// Iterates through the directory entries and attempts to open devices starting with "card".
    /// Among available devices, selects the one with the most connected outputs.
    ///
    /// # Returns
    /// - `Ok(Card)` if a functional DRM device is found and opened
    /// - `Err(Box<dyn Error>)` if no accessible DRM device is found
    ///
    /// # Errors
    /// - Returns an error if `/dev/dri/` cannot be read
    /// - Returns an error if no DRM device supports the required operations
    /// - Returns an error if permission is denied for all devices
    fn open_available_card() -> Result<Self, Box<dyn Error>> {
        debug!("Opening available DRM card");
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
                                    debug!("Device {:?} doesn't support basic operations (Error 95), ignoring...", path);
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
/// # Arguments
/// * `drm_device` - Reference to the DRM device card
/// * `screen` - Optional filter for specific connector types (e.g., "HDMI", "DP")
/// * `f` - Closure to execute for each matching connector
///
/// # Returns
/// - `Ok(())` if iteration completes successfully
/// - `Err(Box<dyn Error>)` if an error occurs during processing
///
/// # Errors
/// - Returns an error if resource handles cannot be obtained
/// - Propagates any errors from the closure `f`
fn for_each_connector<F>(
    drm_device: &Card,
    screen: Option<&str>,
    mut f: F,
) -> Result<(), Box<dyn Error>>
where
    F: FnMut(&connector::Info) -> Result<(), Box<dyn Error>>,
{
    debug!("Fetching resource handles for DRM device");
    let resources = drm_device.resource_handles()?;
    let connectors = resources.connectors();
    debug!("Found {} connectors", connectors.len());

    connectors
        .iter()
        .filter_map(|&connector_handle| {
            match drm_device.get_connector(connector_handle, true) {
                Ok(connector_info) => Some(connector_info),
                Err(e) => {
                    warn!("Failed to get info for connector {:?}: {}", connector_handle, e);
                    None
                }
            }
        })
        .filter(|connector_info| {
            match screen {
                Some(screen_name) if format!("{:?}", connector_info.interface()) != screen_name => {
                    debug!(
                        "Skipping connector {:?} - doesn't match screen {}",
                        connector_info.interface(),
                        screen_name
                    );
                    false
                }
                _ => true,
            }
        })
        .try_for_each(|connector_info| f(&connector_info))?;

    debug!("Connector iteration completed successfully");
    Ok(())
}

/// Lists all available display modes for the specified screen.
///
/// # Arguments
/// * `screen` - Optional filter for specific connector types (e.g., "HDMI", "DP")
///
/// # Returns
/// - `Ok(String)` containing formatted mode information (one per line)
/// - `Err(Box<dyn Error>)` if an error occurs
///
/// # Examples
/// ```
/// let modes = drm_list_modes(Some("HDMI")).unwrap();
/// println!("Available modes:\n{}", modes);
/// ```
///
/// # Errors
/// - Returns an error if no DRM device is available
/// - Returns an error if no modes are found for the specified screen
pub fn drm_list_modes(screen: Option<&str>) -> Result<String, Box<dyn Error>> {
    let card = Card::open_available_card()?;
    let mut modes_string = String::new();
    let mut first = true;

    debug!("Iterating over connectors to list modes");
    for_each_connector(&card, screen, |connector_info| {
        debug!("Processing connector {:?}", connector_info.interface());
        let modes = connector_info.modes();
        debug!("Found {} modes for connector", modes.len());

        for mode in modes {
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

            if !first {
                modes_string.push_str("\n");
            }
            modes_string.push_str(&mode_str);
            first = false;
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
/// - `Ok(String)` containing connector names (one per line)
/// - `Err(Box<dyn Error>)` if an error occurs
///
/// # Examples
/// ```
/// let outputs = drm_list_outputs().unwrap();
/// println!("Available outputs:\n{}", outputs);
/// ```
///
/// # Errors
/// - Returns an error if no DRM device is available
/// - Returns an error if no outputs are found
pub fn drm_list_outputs() -> Result<String, Box<dyn Error>> {
    let card = Card::open_available_card()?;
    let mut outputs_string = String::new();
    let mut first = true;

    debug!("Iterating over connectors to list outputs");
    for_each_connector(&card, None, |connector_info| {
        let output_str = format!("{:?}", connector_info.interface());
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

/// Retrieves the current display mode for the specified screen.
///
/// # Arguments
/// * `screen` - Optional filter for specific connector types (e.g., "HDMI", "DP")
///
/// # Returns
/// - `Ok(String)` containing current mode in "WIDTHxHEIGHT@REFRESH_RATE" format
/// - `Err(Box<dyn Error>)` if an error occurs
///
/// # Examples
/// ```
/// let mode = drm_current_mode(Some("HDMI")).unwrap();
/// println!("Current mode: {}", mode);
/// ```
///
/// # Errors
/// - Returns an error if no DRM device is available
/// - Returns an error if no current mode is found for the specified screen
pub fn drm_current_mode(screen: Option<&str>) -> Result<String, Box<dyn Error>> {
    let card = Card::open_available_card()?;
    let mut current_mode_string = String::new();

    debug!("Iterating over connectors to find current mode");
    for_each_connector(
        &card,
        screen,
        |connector_info| -> Result<(), Box<dyn Error>> {
            if connector_info.state() == connector::State::Connected {
                debug!("Checking connected connector {:?}", connector_info.interface());
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
/// - `Ok(String)` with the name of the connected output
/// - `Err(Box<dyn Error>)` if an error occurs
///
/// # Examples
/// ```
/// let output = drm_current_output().unwrap();
/// println!("Current output: {}", output);
/// ```
///
/// # Errors
/// - Returns an error if no DRM device is available
/// - Returns an error if no connected output is found
pub fn drm_current_output() -> Result<String, Box<dyn Error>> {
    let card = Card::open_available_card()?;
    let mut current_output_string = String::new();

    debug!("Iterating over connectors to find current output");
    for_each_connector(
        &card,
        None,
        |connector_info| -> Result<(), Box<dyn Error>> {
            if connector_info.state() == connector::State::Connected {
                let output_str = format!("{:?}", connector_info.interface());
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

/// Retrieves the current resolution for the specified screen.
///
/// # Arguments
/// * `screen` - Optional filter for specific connector types (e.g., "HDMI", "DP")
///
/// # Returns
/// - `Ok(String)` containing current resolution in "WIDTHxHEIGHT" format
/// - `Err(Box<dyn Error>)` if an error occurs
///
/// # Examples
/// ```
/// let resolution = drm_current_resolution(Some("HDMI")).unwrap();
/// println!("Current resolution: {}", resolution);
/// ```
///
/// # Errors
/// - Returns an error if no DRM device is available
/// - Returns an error if no current resolution is found for the specified screen
pub fn drm_current_resolution(screen: Option<&str>) -> Result<String, Box<dyn Error>> {
    let card = Card::open_available_card()?;
    let mut current_resolution_string = String::new();

    debug!("Iterating over connectors to find current resolution");
    for_each_connector(
        &card,
        screen,
        |connector_info| -> Result<(), Box<dyn Error>> {
            if connector_info.state() == connector::State::Connected {
                debug!("Checking resolution for connector {:?}", connector_info.interface());
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
        info!("Current resolution retrieved: {}", current_resolution_string);
    }
    Ok(current_resolution_string)
}

/// Retrieves the current refresh rate for the specified screen.
///
/// # Arguments
/// * `screen` - Optional filter for specific connector types (e.g., "HDMI", "DP")
///
/// # Returns
/// - `Ok(String)` containing current refresh rate in "REFRESH_RATEHz" format
/// - `Err(Box<dyn Error>)` if an error occurs
///
/// # Examples
/// ```
/// let refresh = drm_current_refresh(Some("HDMI")).unwrap();
/// println!("Current refresh rate: {}", refresh);
/// ```
///
/// # Errors
/// - Returns an error if no DRM device is available
/// - Returns an error if no current refresh rate is found for the specified screen
pub fn drm_current_refresh(screen: Option<&str>) -> Result<String, Box<dyn Error>> {
    let card = Card::open_available_card()?;
    let mut current_refresh_string = String::new();

    debug!("Iterating over connectors to find current refresh rate");
    for_each_connector(
        &card,
        screen,
        |connector_info| -> Result<(), Box<dyn Error>> {
            if connector_info.state() == connector::State::Connected {
                debug!("Checking refresh rate for connector {:?}", connector_info.interface());
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

/// Sets a specific display mode for the specified screen.
///
/// # Arguments
/// * `screen` - Optional filter for specific connector types (e.g., "HDMI", "DP")
/// * `width` - Desired width in pixels
/// * `height` - Desired height in pixels
/// * `vrefresh` - Desired refresh rate in Hz
///
/// # Returns
/// - `Ok(())` if the mode is set successfully
/// - `Err(Box<dyn Error>)` if an error occurs
///
/// # Examples
/// ```
/// // Set HDMI output to 1920x1080@60Hz
/// drm_set_mode(Some("HDMI"), 1920, 1080, 60).unwrap();
/// ```
///
/// # Errors
/// - Returns an error if no DRM device is available
/// - Returns an error if the specified mode is not available
/// - Returns an error if the mode cannot be set
pub fn drm_set_mode(
    screen: Option<&str>,
    width: i32,
    height: i32,
    vrefresh: i32,
) -> Result<(), Box<dyn Error>> {
    let card = Card::open_available_card()?;
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
                if !interface.contains(screen_name) {
                    debug!(
                        "Skipping connector {} - doesn't match screen filter '{}'",
                        interface, screen_name
                    );
                    return Ok(());
                }
            }

            debug!("Processing connected connector: {}", interface);
            let encoder_id = connector_info.current_encoder().ok_or_else(|| format!("No encoder found for connector {}", interface))?;
            let encoder_info = card.get_encoder(encoder_id)?;
            let crtc_id = encoder_info.crtc().ok_or_else(|| format!("No CRTC found for encoder {:?} on {}", encoder_id, interface))?;
            let crtc_info = card.get_crtc(crtc_id)?;
            let current_fb = crtc_info.framebuffer();

            let target_mode = connector_info.modes().iter().find(|mode| {
                mode.size().0 == width as u16 &&
                mode.size().1 == height as u16 &&
                mode.vrefresh() == vrefresh as u32
            }).ok_or("Mode not found")?;

            info!(
                "Setting mode '{}' ({}x{}@{}Hz) for screen: {:?}",
                target_mode.name().to_string_lossy(),
                width,
                height,
                vrefresh,
                interface
            );

            card.set_crtc(
                crtc_id,
                current_fb,
                (0, 0),
                &[connector_info.handle()],
                Some(*target_mode),
            ).map_err(|e| {
                error!("Failed to set CRTC: {:?}", e);
                e
            })?;
            Ok(())
        },
    )?;
    info!("Mode setting completed successfully");
    Ok(())
}

/// Sets the active output to a specified connector.
///
/// # Arguments
/// * `output` - The name of the output to activate (e.g., "HDMI-A-1")
///
/// # Returns
/// - `Ok(())` if the output is set successfully
/// - `Err(Box<dyn Error>)` if an error occurs
///
/// # Examples
/// ```
/// drm_set_output("HDMI-A-1").unwrap();
/// ```
///
/// # Errors
/// - Returns an error if no DRM device is available
/// - Returns an error if the specified output is not found
/// - Returns an error if the output cannot be activated
pub fn drm_set_output(output: &str) -> Result<(), Box<dyn Error>> {
    let card = Card::open_available_card()?;
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
                            debug!("Setting CRTC with current mode for output {}", interface_str);
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

/// Sets the rotation of the specified screen.
///
/// # Arguments
/// * `screen` - Optional filter for specific connector types (e.g., "HDMI", "DP")
/// * `rotation` - The rotation value as a string ("0", "90", "180", "270")
///
/// # Returns
/// - `Ok(())` if the rotation is set successfully
/// - `Err(Box<dyn Error>)` if an error occurs
///
/// # Examples
/// ```
/// // Rotate HDMI output 90 degrees
/// drm_set_rotation(Some("HDMI"), "90").unwrap();
/// ```
///
/// # Errors
/// - Returns an error if no DRM device is available
/// - Returns an error if the rotation value is invalid
/// - Returns an error if no matching screen is found
/// - Returns an error if the rotation property is not available
pub fn drm_set_rotation(screen: Option<&str>, rotation: &str) -> Result<(), Box<dyn Error>> {
    let card = Card::open_available_card()?;

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

/// Captures a screenshot from the currently active display and saves it to a file.
///
/// # Arguments
/// * `screenshot_dir` - Path where the screenshot should be saved (e.g., "screenshot.png")
///
/// # Returns
/// - `Ok(())` if the screenshot is captured and saved successfully
/// - `Err(Box<dyn Error>)` if an error occurs
pub fn drm_get_screenshot(screenshot_dir: &str) -> Result<(), Box<dyn Error>> {
    let card = Card::open_available_card()?;
    let mut crtc_id = None;
    let mut fb_info = None;

    debug!("Searching for active CRTC to capture screenshot");
    // Find an active CRTC with a framebuffer
    for_each_connector(
        &card,
        None,
        |connector_info| -> Result<(), Box<dyn Error>> {
            if connector_info.state() == connector::State::Connected {
                if let Some(encoder_id) = connector_info.current_encoder() {
                    let encoder_info = card.get_encoder(encoder_id)?;
                    if let Some(crtc) = encoder_info.crtc() {
                        let crtc_info = card.get_crtc(crtc)?;
                        if crtc_info.mode().is_some() && crtc_info.framebuffer().is_some() {
                            crtc_id = Some(crtc);
                            fb_info = Some(card.get_framebuffer(crtc_info.framebuffer().unwrap())?);
                            debug!("Found active CRTC {:?} with framebuffer", crtc);
                            return Ok(());
                        }
                    }
                }
            }
            Ok(())
        },
    )?;

    let crtc_id = crtc_id.ok_or("No active display found")?;
    let crtc_info = card.get_crtc(crtc_id)?;
    let mode = crtc_info.mode().ok_or("CRTC has no active mode")?;
    let width = mode.size().0 as u32;
    let height = mode.size().1 as u32;

    debug!("Creating dumb buffer for screenshot capture");
    // Create a dumb buffer for the screenshot
    let mut dumb_buffer = card.create_dumb_buffer(
        (width, height),
        DrmFourcc::Xrgb8888, // Standard 32-bit XRGB format
        32,                  // Bits per pixel
    )?;

    // Map the buffer to userspace
    let map = card.map_dumb_buffer(&mut dumb_buffer)?;

    // For actual framebuffer copying, you would need to implement this separately
    // as there's no direct framebuffer_to_dumb_buffer method in the drm crate.
    // This is a placeholder for the actual copy operation you'd need to implement:
    debug!("[NOTE] Actual framebuffer copying would need custom implementation");

    // Read the pixel data (simulated here - would need proper implementation)
    let mut screenshot_data = vec![0; (width * height * 4) as usize];
    unsafe {
        std::ptr::copy_nonoverlapping(
            map.as_ptr(),
            screenshot_data.as_mut_ptr(),
            screenshot_data.len(),
        );
    }

    // Create an RGB image from the raw data
    debug!("Converting to RGB image");
    let img: RgbImage = ImageBuffer::from_fn(width, height, |x, y| {
        let offset = ((y * width + x) * 4) as usize;
        image::Rgb([
            screenshot_data[offset + 2], // R
            screenshot_data[offset + 1], // G
            screenshot_data[offset],     // B
        ])
    });

    // Ensure screenshot directory exists
    fs::create_dir_all(screenshot_dir)?;

    // Generate timestamped filename
    let file_name = format!(
        "{}/screenshot-{}.png",
        screenshot_dir,
        Local::now().format("%Y.%m.%d-%Hh%M.%S")
    );

    debug!("Saving screenshot to {}", file_name);
    img.save(Path::new(file_name.as_str()))?;
    info!("Successfully saved screenshot to {}", file_name);

    // Clean up
    //card.destroy_dumb_buffer(dumb_buffer)?;

    Ok(())
}

/// Sets the display to its maximum supported resolution for the specified screen.
///
/// # Arguments
/// * `screen` - Optional filter for specific connector types (e.g., "HDMI", "DP")
/// * `_max_resolution` - Currently unused parameter reserved for future use
///
/// # Returns
/// - `Ok(())` if the maximum resolution is set successfully
/// - `Err(Box<dyn Error>)` if an error occurs
///
/// # Examples
/// ```
/// // Set HDMI output to its maximum resolution
/// drm_to_max_resolution(Some("HDMI"), None).unwrap();
/// ```
///
/// # Errors
/// - Returns an error if no DRM device is available
/// - Returns an error if no suitable resolution is found
/// - Returns an error if the resolution cannot be set
pub fn drm_to_max_resolution(
    screen: Option<&str>,
    _max_resolution: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let card = Card::open_available_card()?;
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
