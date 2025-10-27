use log::{debug, error, info, warn};
use std::fs::OpenOptions;
use std::os::unix::io::{AsFd, BorrowedFd};
use std::path::Path;

use drm::control::{Device as ControlDevice, connector};
use drm::Device;

use crate::screen::backend::{DisplayBackend, DisplayMode, DisplayOutput, ModeParams, RotationParams};
use crate::utils::error::{RegmsgError, Result};

const DRM_MODE_PATH: &str = "/var/run/drmMode";

/// Represents a DRM (Direct Rendering Manager) device card.
#[derive(Debug)]
pub struct DrmCard(std::fs::File);

impl AsFd for DrmCard {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl Device for DrmCard {}
impl ControlDevice for DrmCard {}

impl DrmCard {
    /// Opens the first available DRM device in `/dev/dri/`.
    pub fn open_available_card() -> Result<Self> {
        debug!("Opening available DRM card");
        let dri_path = Path::new("/dev/dri/");
        info!("Searching for DRM devices in {:?}", dri_path);

        let mut best_card = None;
        let mut max_connectors = 0;

        for entry in dri_path.read_dir()
            .map_err(|e| RegmsgError::SystemError(format!("Failed to read dir: {}", e)))?
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            if let Some(file_name) = path.file_name().map(|f| f.to_string_lossy()) {
                if file_name.starts_with("card") {
                    debug!("Attempting to open device: {:?}", path);
                    let mut options = OpenOptions::new();
                    options.read(true).write(false);

                    match options.open(&path) {
                        Ok(file) => {
                            let card = DrmCard(file);
                            match card.resource_handles() {
                                Ok(resources) => {
                                    let num_connectors = resources.connectors().len();
                                    info!(
                                        "Device {:?} opened with {} connectors.",
                                        path, num_connectors
                                    );

                                    if num_connectors > max_connectors {
                                        best_card = Some(card);
                                        max_connectors = num_connectors;
                                    }
                                }
                                Err(e) if e.raw_os_error() == Some(95) => {
                                    debug!(
                                        "Device {:?} doesn't support basic operations (Error 95), ignoring...",
                                        path
                                    );
                                }
                                Err(e) => {
                                    error!(
                                        "Error obtaining resources from device {:?}: {:?}",
                                        path, e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to open device {:?}: {:?}", path, e);
                        }
                    }
                }
            }
        }

        if let Some(card) = best_card {
            Ok(card)
        } else {
            error!("No functional DRM device found in {:?}.", dri_path);
            Err(RegmsgError::SystemError("No DRM device found or accessible".to_string()))
        }
    }
}

/// Backend implementation for DRM/KMS
pub struct DrmBackend;

impl DrmBackend {
    pub fn new() -> Self {
        Self
    }
    


    /// Helper to iterate over connectors
    fn for_each_connector<F>(&self, screen: Option<&str>, mut f: F) -> Result<()>
    where
        F: FnMut(&connector::Info) -> Result<()>,
    {
        debug!("Fetching resource handles for DRM device");
        let card = DrmCard::open_available_card()?;
        let resources = card.resource_handles()
            .map_err(|e| RegmsgError::BackendError {
                backend: "DRM".to_string(),
                message: e.to_string(),
            })?;
        
        let connectors = resources.connectors();
        debug!("Found {} connectors", connectors.len());

        for &connector_handle in connectors {
            match card.get_connector(connector_handle, true) {
                Ok(connector_info) => {
                    if let Some(screen_name) = screen {
                        if format!("{:?}", connector_info.interface()) != screen_name {
                            debug!(
                                "Skipping connector {:?} - doesn't match screen {}",
                                connector_info.interface(),
                                screen_name
                            );
                            continue;
                        }
                    }
                    
                    f(&connector_info)?;
                }
                Err(e) => {
                    warn!(
                        "Failed to get info for connector {:?}: {}",
                        connector_handle, e
                    );
                    // Continue with other connectors
                }
            }
        }

        debug!("Connector iteration completed successfully");
        Ok(())
    }
}

impl DisplayBackend for DrmBackend {
    fn list_outputs(&self) -> Result<Vec<DisplayOutput>> {
        let mut outputs = Vec::new();
        
        self.for_each_connector(None, |connector_info| {
            let name = format!("{:?}", connector_info.interface());
            let modes = connector_info
                .modes()
                .iter()
                .map(|mode| DisplayMode {
                    width: mode.size().0 as u32,
                    height: mode.size().1 as u32,
                    refresh_rate: mode.vrefresh() as u32,
                    name: format!("{}x{}@{}Hz", mode.size().0, mode.size().1, mode.vrefresh()),
                })
                .collect();

            outputs.push(DisplayOutput {
                name,
                modes,
                current_mode: None, // We'll need to check the current mode separately
                is_connected: connector_info.state() == connector::State::Connected,
                rotation: 0, // Not available directly from connector
            });

            Ok(())
        })?;

        // Update current modes for connected outputs
        for output in &mut outputs {
            if output.is_connected {
                match self.current_mode(Some(&output.name)) {
                    Ok(mode) => output.current_mode = Some(mode),
                    Err(_) => {} // Leave as None if we can't determine current mode
                }
            }
        }

        Ok(outputs)
    }

    fn list_modes(&self, screen: Option<&str>) -> Result<Vec<DisplayMode>> {
        let mut all_modes = Vec::new();
        
        self.for_each_connector(screen, |connector_info| {
            let modes = connector_info
                .modes()
                .iter()
                .map(|mode| DisplayMode {
                    width: mode.size().0 as u32,
                    height: mode.size().1 as u32,
                    refresh_rate: mode.vrefresh() as u32,
                    name: format!("{}x{}@{}Hz", mode.size().0, mode.size().1, mode.vrefresh()),
                })
                .collect::<Vec<_>>();
            
            all_modes.extend(modes);
            Ok(())
        })?;

        Ok(all_modes)
    }

    fn current_mode(&self, screen: Option<&str>) -> Result<DisplayMode> {
        let mut current_mode = None;
        
        self.for_each_connector(screen, |connector_info| {
            if connector_info.state() == connector::State::Connected {
                debug!(
                    "Checking connected connector {:?}",
                    connector_info.interface()
                );
                if let Some(encoder_id) = connector_info.current_encoder() {
                    debug!("Fetching encoder info for ID {:?}", encoder_id);
                    let card = DrmCard::open_available_card()?;
                    let encoder_info = card.get_encoder(encoder_id)
                        .map_err(|e| RegmsgError::BackendError {
                            backend: "DRM".to_string(),
                            message: e.to_string(),
                        })?;
                    if let Some(crtc_id) = encoder_info.crtc() {
                        debug!("Fetching CRTC info for ID {:?}", crtc_id);
                        let crtc_info = card.get_crtc(crtc_id)
                            .map_err(|e| RegmsgError::BackendError {
                                backend: "DRM".to_string(),
                                message: e.to_string(),
                            })?;
                        if let Some(mode) = crtc_info.mode() {
                            current_mode = Some(DisplayMode {
                                width: mode.size().0 as u32,
                                height: mode.size().1 as u32,
                                refresh_rate: mode.vrefresh() as u32,
                                name: format!("{}x{}@{}Hz", mode.size().0, mode.size().1, mode.vrefresh()),
                            });
                        }
                    }
                }
            }
            Ok(())
        })?;

        current_mode.ok_or_else(|| RegmsgError::NotFound("Current mode".to_string()))
    }

    fn current_resolution(&self, screen: Option<&str>) -> Result<(u32, u32)> {
        let mode = self.current_mode(screen)?;
        Ok((mode.width, mode.height))
    }

    fn current_refresh_rate(&self, screen: Option<&str>) -> Result<u32> {
        let mode = self.current_mode(screen)?;
        Ok(mode.refresh_rate)
    }

    fn current_rotation(&self, _screen: Option<&str>) -> Result<u32> {
        // Rotation is not typically handled at the connector level in DRM
        // It's usually handled by the compositor or CRTC properties
        info!("TODO: Implement drm_current_rotation properly");
        Ok(0)
    }

    fn set_mode(&self, screen: Option<&str>, mode_params: &ModeParams) -> Result<()> {
        let _card = DrmCard::open_available_card()?;
        
        debug!("Iterating over connectors to set display mode");
        
        // Find a matching connector and update it
        self.for_each_connector(screen, |connector_info| {
            // Skip disconnected outputs
            if connector_info.state() != connector::State::Connected {
                return Ok(());
            }

            // Get the connector interface as a string (e.g., "HDMI-A-1")
            let interface = format!("{:?}", connector_info.interface());

            // If a screen name filter is specified, skip non-matching connectors
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

            // Search for a matching mode with the requested resolution and refresh rate
            let target_mode = connector_info
                .modes()
                .iter()
                .find(|mode| {
                    mode.size().0 == mode_params.width as u16
                        && mode.size().1 == mode_params.height as u16
                        && mode.vrefresh() == mode_params.refresh_rate as u32
                });

            if let Some(target_mode) = target_mode {
                // Write the mode string to a system state file (used by some services/tools)
                let mode_str = format!("{}x{}@{}", mode_params.width, mode_params.height, mode_params.refresh_rate);
                std::fs::write(DRM_MODE_PATH, &mode_str)
                    .map_err(|e| RegmsgError::SystemError(format!("Failed to write to DRM mode path: {}", e)))?;
                debug!("Writing mode string to {}: {}", DRM_MODE_PATH, mode_str);

                info!(
                    "Setting mode '{}' ({}x{}@{}Hz) for screen: {:?}",
                    target_mode.name().to_string_lossy(),
                    mode_params.width,
                    mode_params.height,
                    mode_params.refresh_rate,
                    interface
                );
            } else {
                warn!("Mode {}x{}@{} not found for output {}", 
                    mode_params.width, mode_params.height, mode_params.refresh_rate, interface);
            }

            Ok(())
        })?;

        info!("Mode setting completed successfully");
        Ok(())
    }

    fn set_rotation(&self, _screen: Option<&str>, _rotation: &RotationParams) -> Result<()> {
        info!("TODO: Implement drm_set_rotation");
        // This is complex in DRM and typically done at compositor level
        Err(RegmsgError::BackendError {
            backend: "DRM".to_string(),
            message: "Rotation not supported at DRM level directly".to_string(),
        })
    }

    fn set_max_resolution(&self, _screen: Option<&str>, _max_resolution: Option<&str>) -> Result<()> {
        info!("TODO: Implement drm_to_max_resolution");
        // Find and set highest available resolution up to max_resolution
        Ok(())
    }

    fn take_screenshot(&self, _screenshot_dir: &str) -> Result<String> {
        info!("TODO: Implement drm_get_screenshot");
        // DRM doesn't provide screenshot functionality directly
        Err(RegmsgError::BackendError {
            backend: "DRM".to_string(),
            message: "Screenshot not supported at DRM level directly".to_string(),
        })
    }

    fn map_touchscreen(&self) -> Result<()> {
        info!("No touchscreen support for DRM backend");
        Ok(())
    }

    fn backend_name(&self) -> &'static str {
        "DRM"
    }
}