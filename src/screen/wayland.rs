use swayipc::{Connection, Output, Mode};
use std::process::Command;
use chrono::Local;
use std::fs;
use log::{info, warn, error, debug};

/// Lists available display modes for a specific screen (or all screens if none specified).
///
/// # Arguments
/// * `screen` - An optional screen name to filter the modes for a specific screen.
///
/// # Returns
/// A `Result` containing a formatted string with the available modes or an error.
pub fn wayland_list_modes(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    info!("Listing display modes for screen: {:?}", screen);
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut modes_string = String::new();

    for output in outputs {
        // If a specific screen was provided, filter by that screen
        if let Some(screen_name) = screen {
            if output.name != screen_name {
                debug!("Skipping output {} as it does not match the specified screen.", output.name);
                continue; // Skip outputs that do not match the screen name
            }
        }

        // Add predefined modes (example modes)
        modes_string.push_str(&format!("{}\n", "max-1920x1080:maximum 1920x1080"));
        modes_string.push_str(&format!("{}\n", "max-640x480:maximum 640x480"));

        // Adds the current output mode if available
        if let Some(current_mode) = output.current_mode {
            modes_string.push_str(&format!(
                "{}x{}@{}:{}x{}@{}Hz\n",
                current_mode.width, current_mode.height, current_mode.refresh,
                current_mode.width, current_mode.height, current_mode.refresh / 1000
            ));
        }

        // Add all available output modes
        for mode in output.modes {
            modes_string.push_str(&format!(
                "{}x{}@{}:{}x{}@{}Hz\n",
                mode.width, mode.height, mode.refresh,
                mode.width, mode.height, mode.refresh / 1000
            ));
        }
    }

    // Prints the display modes (optional, depending on usage)
    print!("{}", modes_string);
    info!("Display modes listed successfully.");
    // Returns the string with the display modes
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
/// # Arguments
/// * `screen` - An optional screen name to filter the current mode for a specific screen.
///
/// # Returns
/// A `Result` containing a formatted string with the current mode or an error.
pub fn wayland_current_mode(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    info!("Getting current display mode for screen: {:?}", screen);
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut current_mode_string = String::new();

    for output in outputs {
        // If a specific screen was provided, filter by that screen
        if let Some(screen_name) = screen {
            if output.name != screen_name {
                debug!("Skipping output {} as it does not match the specified screen.", output.name);
                continue; // Skip outputs that do not match the screen name
            }
        }

        if let Some(current_mode) = output.current_mode {
            current_mode_string.push_str(&format!(
                "{}x{}@{}Hz\n",
                current_mode.width, current_mode.height, current_mode.refresh
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
/// # Arguments
/// * `screen` - An optional screen name to filter the resolution for a specific screen.
///
/// # Returns
/// A `Result` containing a formatted string with the current resolution or an error.
pub fn wayland_current_resolution(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    info!("Getting current resolution for screen: {:?}", screen);
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut resolution_string = String::new();

    for output in outputs {
        // If a specific screen was provided, filter by that screen
        if let Some(screen_name) = screen {
            if output.name != screen_name {
                debug!("Skipping output {} as it does not match the specified screen.", output.name);
                continue; // Skip outputs that do not match the screen name
            }
        }

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
/// # Arguments
/// * `screen` - An optional screen name to filter the refresh rate for a specific screen.
///
/// # Returns
/// A `Result` containing a formatted string with the current refresh rate or an error.
pub fn wayland_current_refresh(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    info!("Getting current refresh rate for screen: {:?}", screen);
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut refresh_string = String::new();

    for output in outputs {
        // If a specific screen was provided, filter by that screen
        if let Some(screen_name) = screen {
            if output.name != screen_name {
                debug!("Skipping output {} as it does not match the specified screen.", output.name);
                continue; // Skip outputs that do not match the screen name
            }
        }

        if let Some(current_mode) = output.current_mode {
            refresh_string.push_str(&format!(
                "{}Hz\n",
                current_mode.refresh / 1000
            ));
        }
    }

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
/// # Arguments
/// * `screen` - An optional screen name to set the mode for a specific screen.
/// * `width` - The width of the resolution.
/// * `height` - The height of the resolution.
/// * `vrefresh` - The vertical refresh rate in Hz.
///
/// # Returns
/// A `Result` indicating success or an error.
pub fn wayland_set_mode(screen: Option<&str>, width: i32, height: i32, vrefresh: i32) -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting display mode to {}x{}@{}Hz for screen: {:?}", width, height, vrefresh, screen);
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    // Iterate over all outputs and attempt to set the display mode
    for output in outputs {
        // If a specific screen was provided, filter by that screen
        if let Some(screen_name) = screen {
            if output.name != screen_name {
                debug!("Skipping output {} as it does not match the specified screen.", output.name);
                continue; // Skip outputs that do not match the screen name
            }
        }

        // Check if the requested display mode is available for the output
        let mode_exists = output.modes.iter().any(|mode| {
            mode.width == width && mode.height == height
        });

        if !mode_exists {
            println!(
                "Mode {}x{}@{}Hz is not available for output '{}'",
                width, height, vrefresh, output.name
            );
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
                )
                .into());
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
/// # Arguments
/// * `screen` - An optional screen name to set the rotation for a specific screen.
/// * `rotation` - The rotation value (0, 90, 180, or 270 degrees).
///
/// # Returns
/// A `Result` indicating success or an error.
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

    // Iterate over all outputs and apply the rotation
    for output in outputs {
        // If a specific screen was provided, filter by that screen
        if let Some(screen_name) = screen {
            if output.name != screen_name {
                debug!("Skipping output {} as it does not match the specified screen.", output.name);
                continue; // Skip outputs that do not match the screen name
            }
        }

        // Send an IPC command to set the rotation for the output
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
/// # Returns
/// A `Result` indicating success or an error.
pub fn wayland_map_touch_screen() -> Result<(), Box<dyn std::error::Error>> {
    info!("Mapping touchscreen to output.");
    // Establish connection to Wayland server
    let mut connection = Connection::new()?;

    // Get the list of inputs (input devices)
    let inputs = connection.get_inputs()?;

    // Find the touchscreen identifier
    let touchscreen = inputs.iter()
        .find(|input| input.input_type == "touch")
        .map(|input| &input.identifier);

    if let Some(touchscreen) = touchscreen {
        let outputs = connection.get_outputs()?;

        // Find the focused output
        let focused_output = outputs.iter()
            .find(|output| output.focused)
            .map(|output| &output.name);

        if let Some(output) = focused_output {
            // Send an IPC command to map the touchscreen to the output
            let command = format!("input {} map_to_output {}", touchscreen, output);
            connection.run_command(&command)?;
            info!("Mapped {} to {}", touchscreen, output);
        }
    } else {
        warn!("No touchscreen found.");
    }

    Ok(())
}

/// Sets the resolution to the maximum allowed within a specified limit.
///
/// # Arguments
/// * `screen` - An optional screen name to set the resolution for a specific screen.
/// * `max_resolution` - An optional maximum resolution limit (e.g., "1920x1080").
///
/// # Returns
/// A `Result` indicating success or an error.
pub fn wayland_min_to_max_resolution(screen: Option<&str>, max_resolution: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting resolution to maximum within limit for screen: {:?}", screen);
    // Parse the maximum resolution if provided, otherwise default to 1920x1080
    let (max_width, max_height) = if let Some(res) = max_resolution {
        let parts: Vec<&str> = res.split('x').collect();
        if parts.len() == 2 {
            let width = parts[0].parse::<i32>().unwrap_or(1920);
            let height = parts[1].parse::<i32>().unwrap_or(1080);
            (width, height)
        } else {
            (1920, 1080)
        }
    } else {
        (1920, 1080)
    };

    // Establish a connection to sway via IPC
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    // Find the focused output or the specified screen
    let focused_output = if let Some(screen_name) = screen {
        outputs.iter().find(|output| output.name == screen_name)
    } else {
        outputs.iter().find(|output| output.focused)
    };

    if let Some(output) = focused_output {
        if let Some(current_mode) = &output.current_mode {
            let current_width = current_mode.width;
            let current_height = current_mode.height;

            // Check if the current resolution is already within limits
            if current_width <= max_width && current_height <= max_height {
                info!("Current resolution {}x{} is already within limits.", current_width, current_height);
                return Ok(());
            }

            // Search for the best resolution mode within the limits
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
                        return Err(format!(
                            "Failed to set resolution for output '{}': {}",
                            output.name, error
                        ).into());
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
