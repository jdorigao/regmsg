use swayipc::{Connection, Output, Mode};
use std::process::Command;
use chrono::Local;
use std::fs;
use log::{info, warn, error, debug};

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
        // Value is in mHz, convert to Hz
        format!("{}Hz", refresh / 1000)
    } else {
        // Value is already in Hz, use as is
        format!("{}Hz", refresh)
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
/// * `screen` - An optional screen name to filter by.
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
                debug!("Skipping output {} as it does not match the specified screen.", output.name);
            }
            matches
        })
    })
}

/// Lists available display modes for a specific screen (or all screens if none specified).
///
/// This function retrieves all available display modes from the Wayland server using swayipc,
/// including predefined modes and the current mode, formatting them into a string.
/// It uses lazy filtering via `filter_outputs` to process only the relevant outputs.
/// The refresh rate is processed to ensure correct conversion from mHz to Hz.
///
/// # Arguments
/// * `screen` - An optional screen name to filter the modes for a specific screen.
///
/// # Returns
/// A `Result` containing a formatted string with the available modes or an error.
///
/// # Errors
/// Returns an error if the connection to the Wayland server fails or if no outputs are retrieved.
///
/// # Examples
/// ```
/// let modes = wayland_list_modes(Some("eDP-1"))?;
/// println!("{}", modes);
/// ```
pub fn wayland_list_modes(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    info!("Listing display modes for screen: {:?}", screen);
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut modes_string = String::new();

    // Use the reusable filter_outputs function to process relevant outputs
    for output in filter_outputs(outputs, screen) {
        // Add predefined modes
        modes_string.push_str(&format!("{}\n", "max-1920x1080:maximum 1920x1080"));
        modes_string.push_str(&format!("{}\n", "max-640x480:maximum 640x480"));

        // Add the current mode if available
        if let Some(current_mode) = output.current_mode {
            modes_string.push_str(&format!(
                "{}x{}@{}:{}x{}@{}\n",
                current_mode.width, current_mode.height, current_mode.refresh,
                current_mode.width, current_mode.height, format_refresh(current_mode.refresh)
            ));
        }

        // Add all available modes for the output
        for mode in output.modes {
            modes_string.push_str(&format!(
                "{}x{}@{}:{}x{}@{}\n",
                mode.width, mode.height, mode.refresh,
                mode.width, mode.height, format_refresh(mode.refresh)
            ));
        }
    }

    // Prints the display modes (optional, depending on usage)
    print!("{}", modes_string);
    info!("Display modes listed successfully.");
    Ok(modes_string)
}

/// Lists all available outputs (screens).
///
/// # Returns
/// A `Result` containing a formatted string with the available outputs or an error.
pub fn wayland_list_outputs() -> Result<String, Box<dyn std::error::Error>> {
    info!("Listing all available outputs.");
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut outputs_string = String::new();

    for output in outputs {
        outputs_string.push_str(&format!("{}\n", output.name));
    }
    print!("{}", outputs_string);
    info!("Outputs listed successfully.");
    Ok(outputs_string)
}

/// Gets the current display mode (resolution and refresh rate) for a specific screen (or all screens).
///
/// This function retrieves the current mode for the specified screen (or all screens if none specified),
/// using `filter_outputs` to target relevant outputs.
///
/// # Arguments
/// * `screen` - An optional screen name to filter the current mode for a specific screen.
///
/// # Returns
/// A `Result` containing a formatted string with the current mode or an error.
///
/// # Errors
/// Returns an error if the connection to the Wayland server fails.
///
/// # Examples
/// ```
/// let mode = wayland_current_mode(Some("eDP-1"))?;
/// println!("{}", mode); // Outputs something like "1920x1080@60Hz\n"
/// ```
pub fn wayland_current_mode(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    info!("Getting current display mode for screen: {:?}", screen);
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut current_mode_string = String::new();

    // Use filter_outputs to process relevant outputs
    for output in filter_outputs(outputs, screen) {
        if let Some(current_mode) = output.current_mode {
            current_mode_string.push_str(&format!(
                "{}x{}@{}\n",
                current_mode.width, current_mode.height, format_refresh(current_mode.refresh)
            ));
        }
    }

    if current_mode_string.is_empty() {
        warn!("No current mode found for the specified screen.");
        current_mode_string.push_str("No current mode found.\n");
    } else {
        info!("Current mode retrieved successfully.");
    }
    print!("{}", current_mode_string);
    Ok(current_mode_string)
}

/// Gets the current active output (screen).
///
/// # Returns
/// A `Result` containing a formatted string with the current active output or an error.
pub fn wayland_current_output() -> Result<String, Box<dyn std::error::Error>> {
    info!("Getting current active output.");
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut current_output_string = String::new();

    for output in outputs {
        if let Some(_current_mode) = output.current_mode {
            current_output_string.push_str(&format!("{}\n", output.name));
        }
    }

    if current_output_string.is_empty() {
        warn!("No current output found.");
        current_output_string.push_str("No current output found.\n");
    } else {
        info!("Current output retrieved successfully.");
    }
    print!("{}", current_output_string);
    Ok(current_output_string)
}

/// Gets the current resolution for a specific screen (or all screens).
///
/// This function retrieves the current resolution for the specified screen (or all screens if none specified),
/// using `filter_outputs` to process only relevant outputs.
///
/// # Arguments
/// * `screen` - An optional screen name to filter the resolution for a specific screen.
///
/// # Returns
/// A `Result` containing a formatted string with the current resolution or an error.
///
/// # Errors
/// Returns an error if the connection to the Wayland server fails.
///
/// # Examples
/// ```
/// let resolution = wayland_current_resolution(Some("eDP-1"))?;
/// println!("{}", resolution); // Outputs something like "1920x1080\n"
/// ```
pub fn wayland_current_resolution(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    info!("Getting current resolution for screen: {:?}", screen);
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut resolution_string = String::new();

    // Use filter_outputs to process relevant outputs
    for output in filter_outputs(outputs, screen) {
        if let Some(current_mode) = output.current_mode {
            resolution_string.push_str(&format!(
                "{}x{}\n",
                current_mode.width, current_mode.height
            ));
        }
    }

    if resolution_string.is_empty() {
        warn!("No current resolution found for the specified screen.");
        resolution_string.push_str("No current resolution found.\n");
    } else {
        info!("Current resolution retrieved successfully.");
    }
    print!("{}", resolution_string);
    Ok(resolution_string)
}

/// Gets the current refresh rate for a specific screen (or all screens).
///
/// This function retrieves the current refresh rate for the specified screen (or all active screens
/// if none is specified) from the Wayland server, using `filter_outputs` for lazy filtering.
/// The refresh rate is converted from mHz to Hz as needed.
///
/// # Arguments
/// * `screen` - An optional screen name to filter the refresh rate for a specific screen.
///
/// # Returns
/// A `Result` containing a formatted string with the current refresh rate (e.g., "60Hz\n") or an error.
///
/// # Errors
/// Returns an error if the connection to the Wayland server fails or if no outputs are retrieved.
///
/// # Examples
/// ```
/// let refresh = wayland_current_refresh(Some("eDP-1"))?;
/// println!("{}", refresh); // Outputs something like "60Hz\n"
/// ```
pub fn wayland_current_refresh(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    info!("Getting current refresh rate for screen: {:?}", screen);
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut refresh_string = String::new();

    // Use the reusable filter_outputs function to process relevant outputs
    for output in filter_outputs(outputs, screen) {
        if let Some(current_mode) = output.current_mode {
            refresh_string.push_str(&format_refresh(current_mode.refresh));
            refresh_string.push_str("\n");
        }
    }

    // Handle case where no refresh rate is found
    if refresh_string.is_empty() {
        warn!("No current refresh rate found for the specified screen.");
        refresh_string.push_str("No current refresh rate found.\n");
    } else {
        info!("Current refresh rate retrieved successfully.");
    }
    print!("{}", refresh_string);
    Ok(refresh_string)
}

/// Sets a specific display mode (resolution and refresh rate) for a screen.
///
/// This function sets the display mode for the specified screen (or all screens if none specified)
/// to the given resolution and refresh rate, using `filter_outputs` to target relevant outputs.
///
/// # Arguments
/// * `screen` - An optional screen name to set the mode for a specific screen.
/// * `width` - The width of the resolution.
/// * `height` - The height of the resolution.
/// * `vrefresh` - The vertical refresh rate in Hz.
///
/// # Returns
/// A `Result` indicating success or an error if the mode cannot be set.
///
/// # Errors
/// Returns an error if the connection fails, the mode is unavailable, or the command fails.
///
/// # Examples
/// ```
/// wayland_set_mode(Some("eDP-1"), 1920, 1080, 60)?;
/// ```
pub fn wayland_set_mode(screen: Option<&str>, width: i32, height: i32, vrefresh: i32) -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting display mode to {}x{}@{}Hz for screen: {:?}", width, height, vrefresh, screen);
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    // Use filter_outputs to iterate over relevant outputs
    for output in filter_outputs(outputs, screen) {
        // Check if the requested display mode is available
        let mode_exists = output.modes.iter().any(|mode| {
            mode.width == width && mode.height == height
        });

        if !mode_exists {
            warn!("Mode {}x{}@{}Hz is not available for output '{}'", width, height, vrefresh, output.name);
            continue;
        }

        // Send an IPC command to set the display mode
        let command = format!(
            "output {} mode {}x{}@{}Hz",
            output.name, width, height, vrefresh
        );
        let replies = connection.run_command(&command)?;

        // Check if the command was executed successfully
        for reply in replies {
            if let Err(error) = reply.as_ref() {
                error!("Failed to set mode for output '{}': {}", output.name, error);
                return Err(format!(
                    "Failed to set mode for output '{}': {}",
                    output.name, error
                ).into());
            }
        }

        println!(
            "Mode set to {}x{}@{}Hz for output '{}'",
            width, height, vrefresh, output.name
        );
        info!("Mode set to {}x{}@{}Hz for output '{}'", width, height, vrefresh, output.name);
    }

    Ok(())
}

/// Enables a specific output (screen).
///
/// # Arguments
/// * `output` - The name of the output to enable.
///
/// # Returns
/// A `Result` indicating success or an error.
pub fn wayland_set_output(output: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Enabling output: {}", output);
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    // Check if the specified output exists
    let output_exists = outputs.iter().any(|o| o.name == output);
    if !output_exists {
        error!("Output '{}' not found", output);
        return Err(format!("Output '{}' not found", output).into());
    }

    // Send an IPC command to enable the specified output
    let command = format!("output {} enable", output);
    let replies = connection.run_command(&command)?;

    // Check if the command was executed successfully
    for reply in replies {
        if let Err(error) = reply {
            error!("Failed to set output: {}", error);
            return Err(format!("Failed to set output: {}", error).into());
        }
    }
    println!("Output '{}' set successfully", output);
    info!("Output '{}' set successfully", output);
    Ok(())
}

/// Sets the rotation for a specific screen (or all screens).
///
/// This function sets the rotation for the specified screen (or all screens if none specified),
/// using `filter_outputs` to target relevant outputs.
///
/// # Arguments
/// * `screen` - An optional screen name to set the rotation for a specific screen.
/// * `rotation` - The rotation value (0, 90, 180, or 270 degrees).
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
pub fn wayland_set_rotation(screen: Option<&str>, rotation: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting rotation to '{}' for screen: {:?}", rotation, screen);
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    // Check if the given rotation is valid
    let valid_rotations = ["0", "90", "180", "270"];
    if !valid_rotations.contains(&rotation) {
        error!("Invalid rotation: '{}'. Valid options are: {:?}", rotation, valid_rotations);
        return Err(format!("Invalid rotation: '{}'. Valid options are: {:?}", rotation, valid_rotations).into());
    }

    // Use filter_outputs to process relevant outputs
    for output in filter_outputs(outputs, screen) {
        // Send an IPC command to set the rotation
        let command = format!("output {} transform {}", output.name, rotation);
        let replies = connection.run_command(&command)?;

        // Check if the command was executed successfully
        for reply in replies {
            if let Err(error) = reply {
                error!("Failed to set rotation for output '{}': {}", output.name, error);
                return Err(format!("Failed to set rotation for output '{}': {}", output.name, error).into());
            }
        }

        info!("Rotation set to '{}' for output '{}'", rotation, output.name);
    }

    Ok(())
}

/// Captures a screenshot of the current screen.
///
/// # Returns
/// A `Result` indicating success or an error.
pub fn wayland_get_screenshot() -> Result<(), Box<dyn std::error::Error>> {
    info!("Capturing screenshot.");
    //Check if the grim command is available before running it
    if !Command::new("grim").arg("--version").output().is_ok() {
        return Err("grim is not installed or unavailable".into());
    }

    // Establish connection to Wayland server
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    // Get the current output (monitor)
    let output_name = outputs
        .iter()
        .find(|output| output.current_mode.is_some())
        .map(|output| &output.name)
        .ok_or("No active output found")?;

    // Ensure the screenshot directory exists
    let screenshot_dir = "/userdata/screenshots";
    fs::create_dir_all(screenshot_dir)?;

    // Generate file name based on current date and time
    let file_name = format!(
        "{}/screenshot-{}.png",
        screenshot_dir,
        Local::now().format("%Y.%m.%d-%Hh%M.%S")
    );

    // Run the `grim` command to capture the screen
    let grim_output = Command::new("grim")
        .arg("-o")
        .arg(output_name)
        .arg(&file_name)
        .output()?;

    // Check if the command was successful
    if !grim_output.status.success() {
        let error_message = String::from_utf8_lossy(&grim_output.stderr);
        error!("Failed to capture screen: {}", error_message);
        return Err(format!("Failed to capture screen: {}", error_message).into());
    }

    println!("Screenshot saved in: {}", file_name);
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
/// // Map the touchscreen to the currently focused output
/// wayland_map_touch_screen()?;
/// ```
pub fn wayland_map_touch_screen() -> Result<(), Box<dyn std::error::Error>> {
    info!("Mapping touchscreen to output.");
    // Establish connection to Wayland server
    let mut connection = Connection::new()?;

    // Retrieve the list of input devices
    let inputs = connection.get_inputs()?;

    // Find the touchscreen device by checking input type
    let touchscreen = inputs.iter()
        .find(|input| input.input_type == "touch")
        .map(|input| &input.identifier);

    // Check if a touchscreen was found
    let touchscreen_id = match touchscreen {
        Some(id) => id,
        None => {
            warn!("No touchscreen device found.");
            return Err("No touchscreen device found".into());
        }
    };

    // Retrieve the list of outputs
    let outputs = connection.get_outputs()?;

    // Find the focused output among available outputs
    let focused_output = outputs.iter()
        .find(|output| output.focused)
        .map(|output| &output.name);

    // Check if a focused output was found
    let output_name = match focused_output {
        Some(name) => name,
        None => {
            warn!("No focused output found.");
            return Err("No focused output found".into());
        }
    };

    // Send an IPC command to map the touchscreen to the focused output
    let command = format!("input {} map_to_output {}", touchscreen_id, output_name);
    let replies = connection.run_command(&command)?;

    // Check if the command executed successfully
    for reply in replies {
        if let Err(error) = reply {
            error!("Failed to map touchscreen to output '{}': {}", output_name, error);
            return Err(format!("Failed to map touchscreen to output '{}': {}", output_name, error).into());
        }
    }

    info!("Mapped touchscreen '{}' to output '{}'", touchscreen_id, output_name);
    Ok(())
}

/// Sets the resolution to the maximum allowed within a specified limit.
///
/// This function adjusts the resolution of a specified screen (or the focused screen if none specified)
/// to the highest available mode within the given limit, using `filter_outputs` for targeting.
///
/// # Arguments
/// * `screen` - An optional screen name to set the resolution for a specific screen.
/// * `max_resolution` - An optional maximum resolution limit (e.g., "1920x1080").
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
pub fn wayland_min_to_max_resolution(screen: Option<&str>, max_resolution: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting resolution to maximum within limit for screen: {:?}", screen);

    // Parse the maximum resolution or use default
    let (max_width, max_height) = match max_resolution {
        Some(res) => {
            let parts: Vec<&str> = res.split('x').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid resolution format: '{}'. Expected 'WxH'", res).into());
            }
            let width = parts[0].parse::<i32>().map_err(|e| format!("Failed to parse width: {}", e))?;
            let height = parts[1].parse::<i32>().map_err(|e| format!("Failed to parse height: {}", e))?;
            if width <= 0 || height <= 0 {
                return Err(format!("Resolution dimensions must be positive: {}x{}", width, height).into());
            }
            (width, height)
        }
        None => (1920, 1080),
    };

    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    // Use filter_outputs and find the focused output if no screen is specified
    let target_output = if let Some(screen_name) = screen {
        filter_outputs(outputs, Some(screen_name)).next()
    } else {
        outputs.into_iter().find(|output| output.focused)
    };

    if let Some(output) = target_output {
        if let Some(current_mode) = &output.current_mode {
            let current_width = current_mode.width;
            let current_height = current_mode.height;

            if current_width <= max_width && current_height <= max_height {
                info!("Current resolution {}x{} is already within limits.", current_width, current_height);
                return Ok(());
            }

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
                let command = format!(
                    "output {} mode {}x{}@{}Hz",
                    output.name, mode.width, mode.height, mode.refresh
                );
                for reply in connection.run_command(&command)? {
                    if let Err(error) = reply {
                        error!("Failed to set resolution for output '{}': {}", output.name, error);
                        return Err(format!("Failed to set resolution for output '{}': {}", output.name, error).into());
                    }
                }
                info!("Resolution set to {}x{} for output '{}'", mode.width, mode.height, output.name);
                return Ok(());
            }
        }
    }

    warn!("No suitable resolution found within {}x{} limits.", max_width, max_height);
    Ok(())
}
