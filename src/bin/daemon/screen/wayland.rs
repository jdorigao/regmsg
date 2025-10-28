use chrono::Local;
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use swayipc::{Connection, Output};

use crate::screen::backend::{
    DisplayBackend, DisplayMode, DisplayOutput, ModeParams, RotationParams,
};
use crate::utils::error::{RegmsgError, Result};

use tracing::{debug, error, info, warn};

/// Pre-processes a vector of outputs into a HashMap for efficient lookup by name.
fn preprocess_outputs(outputs: Vec<Output>) -> HashMap<String, Output> {
    outputs
        .into_iter()
        .map(|output| (output.name.clone(), output))
        .collect()
}

/// Converts a refresh rate from mHz to Hz if applicable.
fn format_refresh(refresh: i32) -> String {
    if refresh >= 1000 {
        // Value is in mHz, convert to Hz by dividing by 1000
        format!("{}", refresh / 1000)
    } else {
        // Value is already in Hz, append "Hz" unit
        format!("{}", refresh)
    }
}

/// Filters a collection of outputs based on an optional screen name.
fn filter_outputs<'a>(outputs: &'a [Output], screen: Option<&str>) -> impl Iterator<Item = &'a Output> {
    outputs.iter().filter(move |output| {
        screen.map_or(true, |screen_name| {
            let matches = output.name == screen_name;
            if !matches {
                // Log skipped outputs for debugging
                debug!(
                    "Skipping output {} as it does not match the specified screen.",
                    output.name
                );
            }
            matches
        })
    })
}

/// Converts swayipc Output to our DisplayOutput
fn convert_output_sway_to_internal(sway_output: &Output) -> DisplayOutput {
    let modes = sway_output
        .modes
        .iter()
        .map(|mode| DisplayMode {
            width: mode.width as u32,
            height: mode.height as u32,
            refresh_rate: (mode.refresh as f32 / 1000.0).round() as u32, // Convert mHz to Hz
            name: format!(
                "{}x{}@{}Hz",
                mode.width,
                mode.height,
                (mode.refresh as f32 / 1000.0).round() as u32
            ),
        })
        .collect();

    let current_mode = sway_output.current_mode.as_ref().map(|mode| DisplayMode {
        width: mode.width as u32,
        height: mode.height as u32,
        refresh_rate: (mode.refresh as f32 / 1000.0).round() as u32, // Convert mHz to Hz
        name: format!(
            "{}x{}@{}Hz",
            mode.width,
            mode.height,
            (mode.refresh as f32 / 1000.0).round() as u32
        ),
    });

    DisplayOutput {
        name: sway_output.name.clone(),
        modes,
        current_mode,
        is_connected: true, // swayipc doesn't have a direct connection status, assume connected
        rotation: match &sway_output.transform {
            Some(transform_str) => {
                // Parse transform string to rotation value
                match transform_str.as_str() {
                    "90" | "90°" | "rotated-90" => 90,
                    "180" | "180°" | "rotated-180" => 180,
                    "270" | "270°" | "rotated-270" => 270,
                    _ => 0, // Default to 0 for "normal", "0", etc.
                }
            }
            None => 0,
        },
    }
}

/// Backend implementation for Wayland
pub struct WaylandBackend;

impl WaylandBackend {
    pub fn new() -> Self {
        Self
    }

    /// Helper method to get sway connection
    fn get_connection(&self) -> Result<Connection> {
        Connection::new()
            .map_err(|e| RegmsgError::BackendError {
                backend: "Wayland".to_string(),
                message: format!("Failed to connect to Wayland/Sway: {}", e),
            })
    }
}

impl DisplayBackend for WaylandBackend {
    fn list_outputs(&self) -> Result<Vec<DisplayOutput>> {
        let mut connection = self.get_connection()?;
        let outputs: Vec<Output> =
            connection
                .get_outputs()
                .map_err(|e| RegmsgError::BackendError {
                    backend: "Wayland".to_string(),
                    message: e.to_string(),
                })?;

        let internal_outputs = outputs
            .iter()
            .map(convert_output_sway_to_internal)
            .collect();

        Ok(internal_outputs)
    }

    fn list_modes(&self, screen: Option<&str>) -> Result<Vec<DisplayMode>> {
        let mut connection = self.get_connection()?;
        let outputs: Vec<Output> =
            connection
                .get_outputs()
                .map_err(|e| RegmsgError::BackendError {
                    backend: "Wayland".to_string(),
                    message: e.to_string(),
                })?;

        let all_modes: Vec<DisplayMode> = filter_outputs(&outputs, screen)
            .flat_map(|output| {
                output.modes.iter().map(|mode| DisplayMode {
                    width: mode.width as u32,
                    height: mode.height as u32,
                    refresh_rate: (mode.refresh as f32 / 1000.0).round() as u32, // Convert mHz to Hz
                    name: format!(
                        "{}x{}@{}Hz",
                        mode.width,
                        mode.height,
                        (mode.refresh as f32 / 1000.0).round() as u32
                    ),
                })
            })
            .collect();

        Ok(all_modes)
    }

    fn current_mode(&self, screen: Option<&str>) -> Result<DisplayMode> {
        let mut connection = self.get_connection()?;
        let outputs: Vec<Output> =
            connection
                .get_outputs()
                .map_err(|e| RegmsgError::BackendError {
                    backend: "Wayland".to_string(),
                    message: e.to_string(),
                })?;

        for output in filter_outputs(&outputs, screen) {
            if let Some(current_mode) = &output.current_mode {
                return Ok(DisplayMode {
                    width: current_mode.width as u32,
                    height: current_mode.height as u32,
                    refresh_rate: (current_mode.refresh as f32 / 1000.0).round() as u32, // Convert mHz to Hz
                    name: format!(
                        "{}x{}@{}Hz",
                        current_mode.width,
                        current_mode.height,
                        (current_mode.refresh as f32 / 1000.0).round() as u32
                    ),
                });
            }
        }

        Err(RegmsgError::NotFound("Current mode".to_string()))
    }

    fn current_resolution(&self, screen: Option<&str>) -> Result<(u32, u32)> {
        let mode = self.current_mode(screen)?;
        Ok((mode.width, mode.height))
    }

    fn current_refresh_rate(&self, screen: Option<&str>) -> Result<u32> {
        let mode = self.current_mode(screen)?;
        Ok(mode.refresh_rate)
    }

    fn current_rotation(&self, screen: Option<&str>) -> Result<u32> {
        let mut connection = self.get_connection()?;
        let outputs: Vec<Output> =
            connection
                .get_outputs()
                .map_err(|e| RegmsgError::BackendError {
                    backend: "Wayland".to_string(),
                    message: e.to_string(),
                })?;

        for output in filter_outputs(&outputs, screen) {
            match &output.transform {
                Some(transform_str) => {
                    // Parse transform string to rotation value
                    return match transform_str.as_str() {
                        "90" | "90°" | "rotated-90" => Ok(90),
                        "180" | "180°" | "rotated-180" => Ok(180),
                        "270" | "270°" | "rotated-270" => Ok(270),
                        _ => Ok(0), // Default to 0 for "normal", "0", etc.
                    };
                }
                None => return Ok(0),
            }
        }

        Ok(0)
    }

    fn set_mode(&self, screen: Option<&str>, mode: &ModeParams) -> Result<()> {
        let mut connection = self.get_connection()?;
        let outputs = connection
            .get_outputs()
            .map_err(|e| RegmsgError::BackendError {
                backend: "Wayland".to_string(),
                message: e.to_string(),
            })?;

        // Pre-process outputs into a HashMap for efficient lookup
        let outputs_map = preprocess_outputs(outputs);

        // Determine target outputs based on the screen argument
        let target_outputs: Vec<&Output> = match screen {
            Some(screen_name) => {
                // If a specific screen is provided, look it up directly in the map
                if let Some(output) = outputs_map.get(screen_name) {
                    vec![output]
                } else {
                    // Screen not found, return an error
                    return Err(RegmsgError::NotFound(format!(
                        "Screen '{}' not found",
                        screen_name
                    )));
                }
            }
            None => {
                // No screen specified, target all outputs
                outputs_map.values().collect()
            }
        };

        let mut any_success = false;

        for output in target_outputs {
            // Check if the requested mode exists among available modes
            let mode_exists = output
                .modes
                .iter()
                .any(|m| m.width == mode.width as i32 && m.height == mode.height as i32);

            if !mode_exists {
                // Mode not available, log a warning and skip this output
                warn!(
                    "Mode {}x{}@{}Hz is not available for output '{}'",
                    mode.width, mode.height, mode.refresh_rate, output.name
                );
                continue;
            }

            // Construct the IPC command to set the mode
            let command = format!(
                "output {} mode {}x{}@{}Hz",
                output.name, mode.width, mode.height, mode.refresh_rate
            );

            // Execute the command and handle replies
            let replies =
                connection
                    .run_command(&command)
                    .map_err(|e| RegmsgError::BackendError {
                        backend: "Wayland".to_string(),
                        message: e.to_string(),
                    })?;

            for reply in replies {
                reply.map_err(|e| RegmsgError::BackendError {
                    backend: "Wayland".to_string(),
                    message: e.to_string(),
                })?;
            }

            info!(
                "Mode set to {}x{}@{}Hz for output '{}'",
                mode.width, mode.height, mode.refresh_rate, output.name
            );
            any_success = true;
        }

        if !any_success && screen.is_some() {
            return Err(RegmsgError::BackendError {
                backend: "Wayland".to_string(),
                message: format!(
                    "Failed to set mode {}x{}@{}Hz for specified screen",
                    mode.width, mode.height, mode.refresh_rate
                ),
            });
        }

        Ok(())
    }

    fn set_rotation(&self, screen: Option<&str>, rotation: &RotationParams) -> Result<()> {
        let mut connection = self.get_connection()?;
        let outputs: Vec<Output> =
            connection
                .get_outputs()
                .map_err(|e| RegmsgError::BackendError {
                    backend: "Wayland".to_string(),
                    message: e.to_string(),
                })?;

        // Validate rotation value
        if ![0, 90, 180, 270].contains(&rotation.rotation) {
            return Err(RegmsgError::InvalidArguments(
                "Rotation must be one of: 0, 90, 180, 270".to_string(),
            ));
        }

        // Iterate over filtered outputs
        for output in filter_outputs(&outputs, screen) {
            // Construct and execute the IPC command to set rotation
            let command = format!("output {} transform {}", output.name, rotation.rotation);
            let replies =
                connection
                    .run_command(&command)
                    .map_err(|e| RegmsgError::BackendError {
                        backend: "Wayland".to_string(),
                        message: e.to_string(),
                    })?;

            for reply in replies {
                reply.map_err(|e| RegmsgError::BackendError {
                    backend: "Wayland".to_string(),
                    message: e.to_string(),
                })?;
            }
            info!(
                "Rotation set to '{}' for output '{}'",
                rotation.rotation, output.name
            );
        }

        Ok(())
    }

    fn set_max_resolution(&self, screen: Option<&str>, max_resolution: Option<&str>) -> Result<()> {
        let (max_width, max_height) = match max_resolution {
            Some(res) => {
                let parts: Vec<&str> = res.split('x').collect();
                if parts.len() != 2 {
                    return Err(RegmsgError::InvalidArguments(format!(
                        "Invalid resolution format: '{}'. Expected 'WxH'",
                        res
                    )));
                }
                let width = parts[0].parse::<u32>().map_err(|e| {
                    RegmsgError::ParseError(format!("Failed to parse width: {}", e))
                })?;
                let height = parts[1].parse::<u32>().map_err(|e| {
                    RegmsgError::ParseError(format!("Failed to parse height: {}", e))
                })?;
                if width == 0 || height == 0 {
                    return Err(RegmsgError::InvalidArguments(format!(
                        "Resolution dimensions must be positive: {}x{}",
                        width, height
                    )));
                }
                (width, height)
            }
            None => (1920, 1080),
        };

        let mut connection = self.get_connection()?;
        let outputs: Vec<Output> =
            connection
                .get_outputs()
                .map_err(|e| RegmsgError::BackendError {
                    backend: "Wayland".to_string(),
                    message: e.to_string(),
                })?;

        // Determine target output (specified screen or focused output)
        let target_output = if let Some(screen_name) = screen {
            filter_outputs(&outputs, Some(screen_name)).next()
        } else {
            outputs.iter().find(|output| output.focused)
        };

        if let Some(output) = target_output {
            if let Some(current_mode) = &output.current_mode {
                let current_width = current_mode.width as u32;
                let current_height = current_mode.height as u32;

                // Check if current resolution is already within limits
                if current_width <= max_width && current_height <= max_height {
                    info!(
                        "Current resolution {}x{} is already within limits.",
                        current_width, current_height
                    );
                    return Ok(());
                }

                // Find the best mode within the specified limits
                let best_mode = output
                    .modes
                    .iter()
                    .filter(|mode| {
                        mode.width as u32 <= max_width && mode.height as u32 <= max_height
                    })
                    .max_by_key(|mode| (mode.width as u32) * (mode.height as u32));

                if let Some(mode) = best_mode {
                    let command = format!(
                        "output {} mode {}x{}@{}Hz",
                        output.name,
                        mode.width,
                        mode.height,
                        format_refresh(mode.refresh)
                    );

                    for reply in
                        connection
                            .run_command(&command)
                            .map_err(|e| RegmsgError::BackendError {
                                backend: "Wayland".to_string(),
                                message: e.to_string(),
                            })?
                    {
                        reply.map_err(|e| RegmsgError::BackendError {
                            backend: "Wayland".to_string(),
                            message: e.to_string(),
                        })?;
                    }
                    info!(
                        "Resolution set to {}x{} for output '{}'",
                        mode.width, mode.height, output.name
                    );
                    return Ok(());
                }
            }
        }

        warn!(
            "No suitable resolution found within {}x{} limits.",
            max_width, max_height
        );
        Ok(())
    }

    fn take_screenshot(&self, screenshot_dir: &str) -> Result<String> {
        info!("Capturing screenshot.");

        // Check if `grim` is available
        if !Command::new("grim")
            .output()
            .map_err(|e| RegmsgError::SystemError(e.to_string()))?
            .status
            .success()
        {
            return Err(RegmsgError::SystemError(
                "grim is not installed or unavailable".to_string(),
            ));
        }

        let mut connection = self.get_connection()?;
        let outputs: Vec<Output> =
            connection
                .get_outputs()
                .map_err(|e| RegmsgError::BackendError {
                    backend: "Wayland".to_string(),
                    message: e.to_string(),
                })?;

        // Find the active output
        let output_name = outputs
            .iter()
            .find(|output| output.current_mode.is_some())
            .map(|output| &output.name)
            .ok_or_else(|| RegmsgError::NotFound("No active output found".to_string()))?;

        // Ensure screenshot directory exists
        fs::create_dir_all(screenshot_dir).map_err(|e| RegmsgError::SystemError(e.to_string()))?;

        // Generate timestamped filename
        let file_name = format!(
            "{}/screenshot-{}.png",
            screenshot_dir,
            Local::now().format("%Y.%m.%d-%Hh%M.%S")
        );

        // Execute `grim` to capture the screenshot
        let grim_output = Command::new("grim")
            .arg("-o")
            .arg(output_name)
            .arg(&file_name)
            .output()
            .map_err(|e| RegmsgError::SystemError(e.to_string()))?;

        if !grim_output.status.success() {
            let error_message = String::from_utf8_lossy(&grim_output.stderr);
            error!("Failed to capture screen: {}", error_message);
            return Err(RegmsgError::SystemError(format!(
                "Failed to capture screen: {}",
                error_message
            )));
        }
        info!("Screenshot saved in: {}", file_name);
        Ok(file_name)
    }

    fn map_touchscreen(&self) -> Result<()> {
        let mut connection = self.get_connection()?;

        // Get list of input devices
        let inputs = connection
            .get_inputs()
            .map_err(|e| RegmsgError::BackendError {
                backend: "Wayland".to_string(),
                message: e.to_string(),
            })?;

        // Find touchscreen device
        let touchscreen = inputs
            .iter()
            .find(|input| input.input_type == "touch")
            .map(|input| &input.identifier);

        let touchscreen_id = match touchscreen {
            Some(id) => id,
            None => {
                return Err(RegmsgError::NotFound(
                    "No touchscreen device found".to_string(),
                ));
            }
        };

        // Get list of outputs
        let outputs = connection
            .get_outputs()
            .map_err(|e| RegmsgError::BackendError {
                backend: "Wayland".to_string(),
                message: e.to_string(),
            })?;

        // Find focused output
        let focused_output = outputs
            .iter()
            .find(|output| output.focused)
            .map(|output| &output.name);

        let output_name = match focused_output {
            Some(name) => name,
            None => {
                return Err(RegmsgError::NotFound("No focused output found".to_string()));
            }
        };

        // Construct and execute IPC command to map touchscreen
        let command = format!("input {} map_to_output {}", touchscreen_id, output_name);
        let replies = connection
            .run_command(&command)
            .map_err(|e| RegmsgError::BackendError {
                backend: "Wayland".to_string(),
                message: e.to_string(),
            })?;

        for reply in replies {
            reply.map_err(|e| RegmsgError::BackendError {
                backend: "Wayland".to_string(),
                message: e.to_string(),
            })?;
        }

        info!(
            "Mapped touchscreen '{}' to output '{}'",
            touchscreen_id, output_name
        );
        Ok(())
    }

    fn backend_name(&self) -> &'static str {
        "Wayland"
    }
}
