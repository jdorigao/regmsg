use chrono::Local;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use swayipc::{Connection, Mode, Output};

/// Pre-processes a vector of outputs into a HashMap for efficient lookup by name.
///
/// This function creates a mapping from output names to their corresponding `Output` objects,
/// allowing O(1) access instead of iterating over the list repeatedly.
///
/// # Arguments
/// * `outputs` - A vector of `Output` objects to preprocess.
///
/// # Returns
/// A `HashMap` where the key is the output name (`String`) and the value is the `Output` object.
///
/// # Performance
/// This runs in O(n) time to build the map, but enables constant-time lookups later.
fn preprocess_outputs(outputs: Vec<Output>) -> HashMap<String, Output> {
    outputs
        .into_iter()
        .map(|output| (output.name.clone(), output))
        .collect()
}

/// Converts a refresh rate from mHz to Hz if applicable.
///
/// This function assumes the refresh rate is in millihertz (mHz) if the value is >= 1000,
/// converting it to hertz (Hz) by dividing by 1000. Otherwise, it assumes the value is already in Hz.
///
/// # Arguments
/// * `refresh` - The refresh rate value to format (in mHz or Hz).
///
/// # Returns
/// A string representing the refresh rate in Hz (e.g., "60Hz").
///
/// # Examples
/// ```
/// assert_eq!(format_refresh(60000), "60Hz"); // Converts from mHz
/// assert_eq!(format_refresh(60), "60Hz");    // Keeps as Hz
/// ```
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
///
/// This function takes a vector of outputs and an optional screen name, returning an iterator
/// over the outputs that match the specified screen name. If no screen name is provided,
/// all outputs are included. Non-matching outputs are logged for debugging purposes.
///
/// # Arguments
/// * `outputs` - A vector of `Output` objects to filter.
/// * `screen` - An optional screen name to filter by (e.g., "eDP-1").
///
/// # Returns
/// An iterator over the filtered `Output` objects.
///
/// # Examples
/// ```
/// let outputs = vec![Output { name: "eDP-1".to_string(), ..Default::default() }];
/// let filtered = filter_outputs(outputs, Some("eDP-1")).collect::<Vec<_>>();
/// assert_eq!(filtered.len(), 1);
/// let filtered = filter_outputs(outputs, None).collect::<Vec<_>>();
/// assert_eq!(filtered.len(), 1);
/// ```
fn filter_outputs(outputs: Vec<Output>, screen: Option<&str>) -> impl Iterator<Item = Output> {
    outputs.into_iter().filter(move |output| {
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

/// Lists available display modes for a specific screen (or all screens if none specified).
///
/// This function retrieves all available display modes from the Wayland server using `swayipc`,
/// including predefined modes and the current mode, formatting them into a string.
/// It uses lazy filtering via `filter_outputs` to process only the relevant outputs.
/// The refresh rate is processed to ensure correct conversion from mHz to Hz.
///
/// # Arguments
/// * `screen` - An optional screen name to filter the modes for a specific screen (e.g., "eDP-1").
///
/// # Returns
/// A `Result` containing a formatted string with the available modes or an error.
/// Each mode is listed in the format "widthxheight@refresh_rate" (e.g., "1920x1080@60Hz").
///
/// # Errors
/// Returns an error if the connection to the Wayland server fails or if no outputs are retrieved.
///
/// # Examples
/// ```rust
/// let modes = wayland_list_modes(Some("eDP-1"))?;
/// println!("{}", modes); // Outputs available modes like "1920x1080@60Hz"
/// ```
pub fn wayland_list_modes(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    // Establish a new connection to the Wayland server
    let mut connection = Connection::new()?;

    // Retrieve all outputs from the server
    let outputs: Vec<Output> = connection.get_outputs()?;

    // Initialize an empty string to store the formatted modes
    let mut modes_string = String::new();

    // Get the total number of outputs
    let filtered_outputs: Vec<_> = filter_outputs(outputs, screen).collect();
    let total_outputs = filtered_outputs.len();

    // Iterate over the filtered outputs with index
    for (output_idx, output) in filtered_outputs.into_iter().enumerate() {
        // Get the total number of modes for this output
        let total_modes = output.modes.len();

        // Iterate over each mode in the output with an index
        for (mode_idx, mode) in output.modes.iter().enumerate() {
            // Format the mode as "widthxheight@refresh_rate" and append it to the string
            modes_string.push_str(&format!(
                "{}x{}@{}:{} {}x{}@{}Hz",
                mode.width,
                mode.height,
                format_refresh(mode.refresh), // Convert refresh rate from mHz to Hz
                output.name,
                mode.width,
                mode.height,
                format_refresh(mode.refresh), // Convert refresh rate from mHz to Hz
            ));

            // Add a newline after each mode except the last one in this output
            if mode_idx < total_modes - 1 {
                modes_string.push_str("\n");
            }
        }

        // Add a newline between outputs except after the last one
        if output_idx < total_outputs - 1 {
            modes_string.push_str("\n");
        }
    }

    // Log successful completion of the operation
    info!("Display modes listed successfully.");

    // Return the formatted string wrapped in a Result
    Ok(modes_string)
}

/// Lists all available outputs (screens).
///
/// This function retrieves all outputs from the Wayland server and formats them into a string.
///
/// # Returns
/// A `Result` containing a formatted string with the available outputs (e.g., "eDP-1\nHDMI-1") or an error.
///
/// # Errors
/// Returns an error if the connection to the Wayland server fails.
///
/// # Examples
/// ```
/// let outputs = wayland_list_outputs()?;
/// println!("{}", outputs); // Outputs something like "eDP-1\nHDMI-1"
/// ```
pub fn wayland_list_outputs() -> Result<String, Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut outputs_string = String::new();

    // Iterate over outputs and build the string
    for (i, output) in outputs.iter().enumerate() {
        outputs_string.push_str(&format!("{}", output.name));

        if i < outputs.len() - 1 {
            // Add a newline except after the last output
            outputs_string.push_str("\n");
        }
    }
    info!("Outputs listed successfully.");
    Ok(outputs_string)
}

/// Gets the current display mode (resolution and refresh rate) for a specific screen (or all screens).
///
/// This function retrieves the current mode for the specified screen (or all screens if none specified),
/// using `filter_outputs` to target relevant outputs.
///
/// # Arguments
/// * `screen` - An optional screen name to filter the current mode for a specific screen (e.g., "eDP-1").
///
/// # Returns
/// A `Result` containing a formatted string with the current mode (e.g., "1920x1080@60Hz") or an error.
///
/// # Errors
/// Returns an error if the connection to the Wayland server fails.
///
/// # Examples
/// ```
/// let mode = wayland_current_mode(Some("eDP-1"))?;
/// println!("{}", mode); // Outputs something like "1920x1080@60Hz"
/// ```
pub fn wayland_current_mode(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut current_mode_string = String::new();

    // Iterate over filtered outputs
    for output in filter_outputs(outputs, screen) {
        if let Some(current_mode) = output.current_mode {
            // Format the current mode with resolution and refresh rate
            current_mode_string.push_str(&format!(
                "{}x{}@{}",
                current_mode.width,
                current_mode.height,
                format_refresh(current_mode.refresh) // Convert refresh rate from mHz to Hz
            ));
        }
    }

    if current_mode_string.is_empty() {
        // Warn if no mode is found and provide a fallback message
        warn!("No current mode found for the specified screen.");
        current_mode_string.push_str("No current mode found.");
    } else {
        info!("Current mode retrieved successfully.");
    }
    Ok(current_mode_string)
}

/// Gets the current active output (screen).
///
/// This function identifies the currently active output based on the presence of a current mode.
///
/// # Returns
/// A `Result` containing a formatted string with the current active output (e.g., "eDP-1") or an error.
///
/// # Errors
/// Returns an error if the connection to the Wayland server fails or no active output is found.
///
/// # Examples
/// ```
/// let output = wayland_current_output()?;
/// println!("{}", output); // Outputs something like "eDP-1"
/// ```
pub fn wayland_current_output() -> Result<String, Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut current_output_string = String::new();

    // Find the output with an active current mode
    for output in outputs {
        if let Some(_current_mode) = output.current_mode {
            current_output_string.push_str(&format!("{}", output.name));
        }
    }

    if current_output_string.is_empty() {
        warn!("No current output found.");
        current_output_string.push_str("No current output found.");
    } else {
        info!("Current output retrieved successfully.");
    }
    Ok(current_output_string)
}

/// Gets the current resolution for a specific screen (or all screens).
///
/// This function retrieves the current resolution for the specified screen (or all screens if none specified),
/// using `filter_outputs` to process only relevant outputs.
///
/// # Arguments
/// * `screen` - An optional screen name to filter the resolution for a specific screen (e.g., "eDP-1").
///
/// # Returns
/// A `Result` containing a formatted string with the current resolution (e.g., "1920x1080") or an error.
///
/// # Errors
/// Returns an error if the connection to the Wayland server fails.
///
/// # Examples
/// ```rust
/// let resolution = wayland_current_resolution(Some("eDP-1"))?;
/// println!("{}", resolution); // Outputs something like "1920x1080"
/// ```
pub fn wayland_current_resolution(
    screen: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut resolution_string = String::new();
    let filtered_outputs: Vec<_> = filter_outputs(outputs, screen).collect(); // Collect to count outputs
    let total_outputs = filtered_outputs.len();

    // Iterate over filtered outputs
    for (i, output) in filtered_outputs.into_iter().enumerate() {
        if let Some(current_mode) = output.current_mode {
            // Format resolution as width x height x name
            resolution_string.push_str(&format!(
                "{}x{}",
                current_mode.width, current_mode.height
            ));
            // Add a newline except after the last output
            if i < total_outputs - 1 {
                resolution_string.push_str("\n");
            }
        }
    }

    if resolution_string.is_empty() {
        warn!("No current resolution found for the specified screen.");
        resolution_string.push_str("No current resolution found.");
    } else {
        info!("Current resolution retrieved successfully.");
    }
    Ok(resolution_string)
}

/// Gets the current refresh rate for a specific screen (or all screens).
///
/// This function retrieves the current refresh rate for the specified screen (or all active screens
/// if none is specified) from the Wayland server, using `filter_outputs` for lazy filtering.
/// The refresh rate is converted from mHz to Hz as needed.
///
/// # Arguments
/// * `screen` - An optional screen name to filter the refresh rate for a specific screen (e.g., "eDP-1").
///
/// # Returns
/// A `Result` containing a formatted string with the current refresh rate (e.g., "60Hz") or an error.
///
/// # Errors
/// Returns an error if the connection to the Wayland server fails or if no outputs are retrieved.
///
/// # Examples
/// ```rust
/// let refresh = wayland_current_refresh(Some("eDP-1"))?;
/// println!("{}", refresh); // Outputs something like "60Hz"
/// ```
pub fn wayland_current_refresh(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut refresh_string = String::new();
    let filtered_outputs: Vec<_> = filter_outputs(outputs, screen).collect(); // Collect to count outputs
    let total_outputs = filtered_outputs.len();

    // Iterate over filtered outputs
    for (i, output) in filtered_outputs.into_iter().enumerate() {
        if let Some(current_mode) = output.current_mode {
            // Format refresh rate using the helper function and append it
            refresh_string.push_str(&format_refresh(current_mode.refresh));
            // Add a newline except after the last output
            if i < total_outputs - 1 {
                refresh_string.push_str("\n");
            }
        }
    }

    if refresh_string.is_empty() {
        warn!("No current refresh rate found for the specified screen.");
        refresh_string.push_str("No current refresh rate found.");
    } else {
        info!("Current refresh rate retrieved successfully.");
    }
    Ok(refresh_string)
}

/// Sets a specific display mode (resolution and refresh rate) for a screen.
///
/// This optimized version uses a `HashMap` to avoid repeated iterations over outputs,
/// improving performance in scenarios with many screens. It targets either a specific screen
/// or all available screens if none is specified.
///
/// # Arguments
/// * `screen` - An optional screen name to set the mode for (e.g., `"eDP-1"`). If `None`, applies to all screens.
/// * `width` - The width of the resolution (e.g., `1920`).
/// * `height` - The height of the resolution (e.g., `1080`).
/// * `vrefresh` - The vertical refresh rate in Hz (e.g., `60`).
///
/// # Returns
/// A `Result` indicating success (`Ok(())`) or an error if the mode cannot be set.
///
/// # Errors
/// Returns an error if:
/// - The connection to the Wayland server fails.
/// - The specified screen is not found.
/// - The requested mode is unavailable for the target screen(s).
/// - The IPC command fails to execute.
///
/// # Examples
/// ```
/// // Set mode for a specific screen
/// wayland_set_mode(Some("eDP-1"), 1920, 1080, 60)?;
/// // Set mode for all screens
/// wayland_set_mode(None, 1280, 720, 60)?;
/// ```
pub fn wayland_set_mode(
    screen: Option<&str>,
    width: i32,
    height: i32,
    vrefresh: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Establish connection to the Wayland server
    let mut connection = Connection::new()?;
    let outputs = connection.get_outputs()?;

    // Pre-process outputs into a HashMap for efficient lookup
    let outputs_map = preprocess_outputs(outputs);

    // Determine target outputs based on the screen argument
    let target_outputs: Vec<&Output> = match screen {
        Some(screen_name) => {
            // If a specific screen is provided, look it up directly in the map
            if let Some(output) = outputs_map.get(screen_name) {
                vec![output]
            } else {
                // Screen not found, log a warning and return an error
                warn!("Screen '{}' not found in available outputs.", screen_name);
                return Err(format!("Screen '{}' not found", screen_name).into());
            }
        }
        None => {
            // No screen specified, target all outputs
            outputs_map.values().collect()
        }
    };

    // Track if at least one output was successfully updated
    let mut any_success = false;

    // Process each target output
    for output in target_outputs {
        // Check if the requested mode exists among available modes
        let mode_exists = output
            .modes
            .iter()
            .any(|mode| mode.width == width && mode.height == height);

        if !mode_exists {
            // Mode not available, log a warning and skip this output
            warn!(
                "Mode {}x{}@{}Hz is not available for output '{}'",
                width, height, vrefresh, output.name
            );
            continue;
        }

        // Construct the IPC command to set the mode
        let command = format!(
            "output {} mode {}x{}@{}Hz",
            output.name, width, height, vrefresh
        );

        // Execute the command and handle replies
        let replies = connection.run_command(&command)?;
        for reply in replies {
            if let Err(error) = reply.as_ref() {
                // Command failed, log the error and propagate it
                error!("Failed to set mode for output '{}': {}", output.name, error);
                return Err(
                    format!("Failed to set mode for output '{}': {}", output.name, error).into(),
                );
            }
        }

        // Mode set successfully, log the result
        info!(
            "Mode set to {}x{}@{}Hz for output '{}'",
            width, height, vrefresh, output.name
        );
        any_success = true;
    }

    // If a specific screen was targeted and no success occurred, return an error
    if !any_success && screen.is_some() {
        return Err(format!(
            "Failed to set mode {}x{}@{}Hz for specified screen",
            width, height, vrefresh
        )
        .into());
    }

    // All operations completed successfully
    Ok(())
}

/// Enables a specific output (screen).
///
/// This function sends an IPC command to enable the specified output.
///
/// # Arguments
/// * `output` - The name of the output to enable (e.g., "eDP-1").
///
/// # Returns
/// A `Result` indicating success or an error.
///
/// # Errors
/// Returns an error if the output is not found or the command fails.
///
/// # Examples
/// ```
/// wayland_set_output("eDP-1")?;
/// ```
pub fn wayland_set_output(output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    // Check if the output exists
    let output_exists = outputs.iter().any(|o| o.name == output);
    if !output_exists {
        error!("Output '{}' not found", output);
        return Err(format!("Output '{}' not found", output).into());
    }

    // Construct and execute the IPC command to enable the output
    let command = format!("output {} enable", output);
    let replies = connection.run_command(&command)?;

    for reply in replies {
        if let Err(error) = reply {
            error!("Failed to set output: {}", error);
            return Err(format!("Failed to set output: {}", error).into());
        }
    }
    info!("Output '{}' set successfully", output);
    Ok(())
}

/// Sets the rotation for a specific screen (or all screens).
///
/// This function sets the rotation for the specified screen (or all screens if none specified),
/// using `filter_outputs` to target relevant outputs.
///
/// # Arguments
/// * `screen` - An optional screen name to set the rotation for a specific screen (e.g., "eDP-1").
/// * `rotation` - The rotation value as a string (e.g., "0", "90", "180", or "270").
///
/// # Returns
/// A `Result` indicating success or an error if the rotation cannot be set.
///
/// # Errors
/// Returns an error if the rotation is invalid or the command fails.
///
/// # Examples
/// ```
/// wayland_set_rotation(Some("eDP-1"), "90")?;
/// ```
pub fn wayland_set_rotation(
    screen: Option<&str>,
    rotation: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    // Validate rotation value
    let valid_rotations = ["0", "90", "180", "270"];
    if !valid_rotations.contains(&rotation) {
        error!(
            "Invalid rotation: '{}'. Valid options are: {:?}",
            rotation, valid_rotations
        );
        return Err(format!(
            "Invalid rotation: '{}'. Valid options are: {:?}",
            rotation, valid_rotations
        )
        .into());
    }

    // Iterate over filtered outputs
    for output in filter_outputs(outputs, screen) {
        // Construct and execute the IPC command to set rotation
        let command = format!("output {} transform {}", output.name, rotation);
        let replies = connection.run_command(&command)?;

        for reply in replies {
            if let Err(error) = reply {
                error!(
                    "Failed to set rotation for output '{}': {}",
                    output.name, error
                );
                return Err(format!(
                    "Failed to set rotation for output '{}': {}",
                    output.name, error
                )
                .into());
            }
        }
        info!(
            "Rotation set to '{}' for output '{}'",
            rotation, output.name
        );
    }

    Ok(())
}

/// Captures a screenshot of the current screen.
///
/// This function uses the `grim` command-line tool to capture a screenshot of the active output
/// and saves it to the `/userdata/screenshots` directory with a timestamped filename.
///
/// # Returns
/// A `Result` indicating success or an error.
///
/// # Errors
/// Returns an error if `grim` is not installed, no active output is found, or the command fails.
///
/// # Examples
/// ```
/// wayland_get_screenshot()?;
/// ```
pub fn wayland_get_screenshot() -> Result<(), Box<dyn std::error::Error>> {
    info!("Capturing screenshot.");
    // Check if `grim` is available
    if !Command::new("grim").arg("--version").output().is_ok() {
        return Err("grim is not installed or unavailable".into());
    }

    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    // Find the active output
    let output_name = outputs
        .iter()
        .find(|output| output.current_mode.is_some())
        .map(|output| &output.name)
        .ok_or("No active output found")?;

    // Ensure screenshot directory exists
    let screenshot_dir = "/userdata/screenshots";
    fs::create_dir_all(screenshot_dir)?;

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
        .output()?;

    if !grim_output.status.success() {
        let error_message = String::from_utf8_lossy(&grim_output.stderr);
        error!("Failed to capture screen: {}", error_message);
        return Err(format!("Failed to capture screen: {}", error_message).into());
    }
    info!("Screenshot saved in: {}", file_name);
    Ok(())
}

/// Maps a touchscreen to a specific output (screen).
///
/// This function identifies a touchscreen device and maps it to the currently focused output
/// using Wayland's IPC commands. If no touchscreen or no focused output is found, it returns an error.
///
/// # Returns
/// A `Result` indicating success or an error if the mapping fails.
///
/// # Errors
/// Returns an error if:
/// - The connection to the Wayland server fails.
/// - No touchscreen device is detected.
/// - No focused output is found among the available outputs.
///
/// # Examples
/// ```
/// wayland_map_touch_screen()?;
/// ```
pub fn wayland_map_touch_screen() -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;

    // Get list of input devices
    let inputs = connection.get_inputs()?;

    // Find touchscreen device
    let touchscreen = inputs
        .iter()
        .find(|input| input.input_type == "touch")
        .map(|input| &input.identifier);

    let touchscreen_id = match touchscreen {
        Some(id) => id,
        None => {
            warn!("No touchscreen device found.");
            return Err("No touchscreen device found".into());
        }
    };

    // Get list of outputs
    let outputs = connection.get_outputs()?;

    // Find focused output
    let focused_output = outputs
        .iter()
        .find(|output| output.focused)
        .map(|output| &output.name);

    let output_name = match focused_output {
        Some(name) => name,
        None => {
            warn!("No focused output found.");
            return Err("No focused output found".into());
        }
    };

    // Construct and execute IPC command to map touchscreen
    let command = format!("input {} map_to_output {}", touchscreen_id, output_name);
    let replies = connection.run_command(&command)?;

    for reply in replies {
        if let Err(error) = reply {
            error!(
                "Failed to map touchscreen to output '{}': {}",
                output_name, error
            );
            return Err(format!(
                "Failed to map touchscreen to output '{}': {}",
                output_name, error
            )
            .into());
        }
    }

    info!(
        "Mapped touchscreen '{}' to output '{}'",
        touchscreen_id, output_name
    );
    Ok(())
}

/// Sets the resolution to the maximum allowed within a specified limit.
///
/// This function adjusts the resolution of a specified screen (or the focused screen if none specified)
/// to the highest available mode within the given limit, using `filter_outputs` for targeting.
///
/// # Arguments
/// * `screen` - An optional screen name to set the resolution for a specific screen (e.g., "eDP-1").
/// * `max_resolution` - An optional maximum resolution limit as a string (e.g., "1920x1080").
///
/// # Returns
/// A `Result` indicating success or an error if the resolution cannot be set.
///
/// # Errors
/// Returns an error if the resolution format is invalid or the command fails.
///
/// # Examples
/// ```
/// wayland_min_to_max_resolution(Some("eDP-1"), Some("1920x1080"))?;
/// ```
pub fn wayland_min_to_max_resolution(
    screen: Option<&str>,
    max_resolution: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse max resolution or use default (1920x1080)
    let (max_width, max_height) = match max_resolution {
        Some(res) => {
            let parts: Vec<&str> = res.split('x').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid resolution format: '{}'. Expected 'WxH'", res).into());
            }
            let width = parts[0]
                .parse::<i32>()
                .map_err(|e| format!("Failed to parse width: {}", e))?;
            let height = parts[1]
                .parse::<i32>()
                .map_err(|e| format!("Failed to parse height: {}", e))?;
            if width <= 0 || height <= 0 {
                return Err(format!(
                    "Resolution dimensions must be positive: {}x{}",
                    width, height
                )
                .into());
            }
            (width, height)
        }
        None => (1920, 1080),
    };

    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    // Determine target output (specified screen or focused output)
    let target_output = if let Some(screen_name) = screen {
        filter_outputs(outputs, Some(screen_name)).next()
    } else {
        outputs.into_iter().find(|output| output.focused)
    };

    if let Some(output) = target_output {
        if let Some(current_mode) = &output.current_mode {
            let current_width = current_mode.width;
            let current_height = current_mode.height;

            // Check if current resolution is already within limits
            if current_width <= max_width && current_height <= max_height {
                info!(
                    "Current resolution {}x{} is already within limits.",
                    current_width, current_height
                );
                return Ok(());
            }

            // Find the best mode within the specified limits
            let mut best_mode: Option<&Mode> = None;
            for mode in &output.modes {
                if mode.width <= max_width && mode.height <= max_height {
                    if let Some(best) = best_mode {
                        if mode.width * mode.height > best.width * best.height {
                            best_mode = Some(mode);
                        }
                    } else {
                        best_mode = Some(mode);
                    }
                }
            }

            if let Some(mode) = best_mode {
                // Set the best available mode
                let command = format!(
                    "output {} mode {}x{}@{}",
                    output.name, mode.width, mode.height, format_refresh(mode.refresh)
                );
                for reply in connection.run_command(&command)? {
                    if let Err(error) = reply {
                        error!(
                            "Failed to set resolution for output '{}': {}",
                            output.name, error
                        );
                        return Err(format!(
                            "Failed to set resolution for output '{}': {}",
                            output.name, error
                        )
                        .into());
                    }
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
