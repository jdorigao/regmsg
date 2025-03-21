use swayipc::{Connection, Output};
use std::process::Command;
use chrono::Local;
use std::fs;

pub fn wayland_list_modes() -> Result<String, Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut modes_string = String::new();

    modes_string.push_str(&format!("{}\n", "max-1920x1080:maximum 1920x1080"));
    modes_string.push_str(&format!("{}\n", "max-640x480:maximum 640x480"));

    for output in outputs {
        if let Some(current_mode) = output.current_mode {
            modes_string.push_str(&format!(
                "{}x{}@{}:{}x{}@{}Hz\n",
                current_mode.width, current_mode.height, current_mode.refresh,
                current_mode.width, current_mode.height, current_mode.refresh / 1000
            ));
        }
        for mode in output.modes {
            modes_string.push_str(&format!(
                "{}x{}@{}:{}x{}@{}Hz\n",
                mode.width, mode.height, mode.refresh,
                mode.width, mode.height, mode.refresh / 1000
            ));
        }
    }
    print!("{}", modes_string);
    Ok(modes_string)
}

pub fn wayland_list_outputs() -> Result<String, Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut outputs_string = String::new();

    for output in outputs {
        outputs_string.push_str(&format!("{}\n", output.name));
    }
    print!("{}", outputs_string);
    Ok(outputs_string)
}


pub fn wayland_current_mode() -> Result<String, Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut current_mode_string = String::new();

    for output in outputs {
        if let Some(current_mode) = output.current_mode {
            current_mode_string.push_str(&format!(
                "{}x{}@{}Hz\n",
                current_mode.width, current_mode.height, current_mode.refresh
            ));
        }
    }

    if current_mode_string.is_empty() {
        current_mode_string.push_str("No current mode found.\n");
    }
    print!("{}", current_mode_string);
    Ok(current_mode_string)
}

pub fn wayland_current_output() -> Result<String, Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut current_output_string = String::new();

    for output in outputs {
        if let Some(_current_mode) = output.current_mode {
            current_output_string.push_str(&format!("{}\n", output.name));
        }
    }

    if current_output_string.is_empty() {
        current_output_string.push_str("No current output found.\n");
    }
    print!("{}", current_output_string);
    Ok(current_output_string)
}

pub fn wayland_current_resolution() -> Result<String, Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut resolution_string = String::new();

    for output in outputs {
        if let Some(current_mode) = output.current_mode {
            resolution_string.push_str(&format!(
                "{}x{}\n",
                current_mode.width, current_mode.height
            ));
        }
    }

    if resolution_string.is_empty() {
        resolution_string.push_str("No current resolution found.\n");
    }
    print!("{}", resolution_string);
    Ok(resolution_string)
}

pub fn wayland_current_refresh() -> Result<String, Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let mut refresh_string = String::new();

    for output in outputs {
        if let Some(current_mode) = output.current_mode {
            refresh_string.push_str(&format!(
                "{}Hz\n",
                current_mode.refresh / 1000
            ));
        }
    }

    if refresh_string.is_empty() {
        refresh_string.push_str("No current refresh rate found.\n");
    }
    print!("{}", refresh_string);
    Ok(refresh_string)
}

pub fn wayland_set_mode(width: i32, height: i32, vrefresh: i32) -> Result<(), Box<dyn std::error::Error>> {
    // Estabelece uma conexão com o sway via IPC
    let mut connection = Connection::new()?;

    // Obtém a lista de outputs (monitores) disponíveis
    let outputs: Vec<Output> = connection.get_outputs()?;

    // Itera sobre todos os outputs e tenta definir o modo de exibição
    for output in outputs {
        let output_name = &output.name;

        // Verifica se o modo de exibição solicitado está disponível para o output
        let mode_exists = output.modes.iter().any(|mode| {
            mode.width == width && mode.height == height
        });

        if !mode_exists {
            println!(
                "Mode {}x{}@{}Hz is not available for output '{}'",
                width, height, vrefresh, output_name
            );
            continue;
        }

        // Envia um comando IPC para definir o modo de exibição
        let command = format!(
            "output {} mode {}x{}@{}Hz",
            output_name, width, height, vrefresh
        );
        let replies = connection.run_command(&command)?;

        // Verifica se o comando foi executado com sucesso
        for reply in replies {
            if reply.is_err() {
                return Err(format!(
                    "Failed to set mode for output '{}': {}",
                    output_name, reply.unwrap_err()
                )
                .into());
            }
        }

        println!(
            "Mode set to {}x{}@{}Hz for output '{}'",
            width, height, vrefresh, output_name
        );
    }

    Ok(())
}

pub fn wayland_set_output(output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    let output_exists = outputs.iter().any(|o| o.name == output);
    if !output_exists {
        return Err(format!("Output '{}' not found", output).into());
    }

    // Sends an IPC command to enable the specified output
    let command = format!("output {} enable", output);
    let replies = connection.run_command(&command)?;

    // Check if the command was executed successfully
    for reply in replies {
        if let Err(error) = reply {
            return Err(format!("Failed to set output: {}", error).into());
        }
    }
    println!("Output '{}' set successfully", output);
    Ok(())
}

pub fn wayland_set_rotation(rotation: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    // Checks if the given rotation is valid
    let valid_rotations = ["0", "90", "180", "270"];
    if !valid_rotations.contains(&rotation) {
        return Err(format!("Invalid rotation: '{}'. Valid options are: {:?}", rotation, valid_rotations).into());
    }

    // Iterates over all outputs and applies rotation
    for output in outputs {
        let output_name = &output.name;

        // Envia um comando IPC para definir a rotação do output
        let command = format!("output {} transform {}", output_name, rotation);
        let replies = connection.run_command(&command)?;

        // Verifica se o comando foi executado com sucesso
        for reply in replies {
            if let Err(error) = reply {
                return Err(format!("Failed to set rotation for output '{}': {}", output_name, error).into());
            }
        }

        println!("Rotation set to '{}' for output '{}'", rotation, output_name);
    }

    Ok(())
}

pub fn wayland_get_screenshot() -> Result<(), Box<dyn std::error::Error>> {
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
        return Err(format!("Failed to capture screen: {}", error_message).into());
    }

    println!("Screenshot saved in: {}", file_name);
    Ok(())
}

pub fn wayland_map_touch_screen() -> Result<(), Box<dyn std::error::Error>> {
    // Establish connection to Wayland server
    let mut connection = Connection::new()?;

    // Gets the list of inputs (input devices)
    let inputs = connection.get_inputs()?;

    // Find the touchscreen identifier
    let touchscreen = inputs.iter()
        .find(|input| input.input_type == "touch")
        .map(|input| &input.identifier);

    if let Some(touchscreen) = touchscreen {
        let outputs = connection.get_outputs()?;

        let focused_output = outputs.iter()
            .find(|output| output.focused)
            .map(|output| &output.name);

        if let Some(output) = focused_output {
            let command = format!("input {} map_to_output {}", touchscreen, output);
            connection.run_command(&command)?;
            println!("Mapped {} to {}", touchscreen, output);
        }
    } else {
        println!("No touchscreen found.");
    }

    Ok(())
}

pub fn wayland_min_to_max_resolution(max_resolution: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
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

    // Establishes a connection to sway via IPC
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    // Find the focused output
    let focused_output = outputs.iter().find(|output| output.focused);

    if let Some(output) = focused_output {
        if let Some(current_mode) = &output.current_mode {
            let current_width = current_mode.width;
            let current_height = current_mode.height;

            // Checks if the current resolution is already within limits
            if current_width <= max_width && current_height <= max_height {
                return Ok(());
            }

            // Search for a resolution mode that is within the limits
            for mode in &output.modes {
                if mode.width <= max_width && mode.height <= max_height {
                    // Sets the new resolution mode
                    let command = format!(
                        "output {} mode {}x{}@{}Hz",
                        output.name, mode.width, mode.height, mode.refresh
                    );
                    connection.run_command(&command)?;
                    println!(
                        "Resolution set to {}x{} for output '{}'",
                        mode.width, mode.height, output.name
                    );
                    return Ok(());
                }
            }
        }
    }

    println!("No suitable resolution found.");
    Ok(())
}
