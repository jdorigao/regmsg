use swayipc::{Connection, Output};
use std::process::Command;
use chrono::Local;
use std::fs;

pub fn wayland_list_modes() -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    for output in outputs {
        if let Some(current_mode) = output.current_mode {
            println!("{}x{}@{}Hz | Current Mode", current_mode.width, current_mode.height, current_mode.refresh / 1000);
        }
        for mode in output.modes {
            println!("{}x{}@{}Hz", mode.width, mode.height, mode.refresh / 1000);
        }
    }

    Ok(())
}

pub fn wayland_list_outputs() -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    for output in outputs {
        println!("{}", output.name);
    }

    Ok(())
}

pub fn wayland_current_mode() -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    for output in outputs {
        if let Some(current_mode) = output.current_mode {
            println!("{}x{}@{}Hz", current_mode.width, current_mode.height, current_mode.refresh / 1000);
        }
    }

    Ok(())
}

pub fn wayland_current_output() -> Result<String, Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    for output in outputs {
        if let Some(_current_mode) = output.current_mode {
            return Ok(output.name)
        }
    }
    Err("No active output found".into())
}

pub fn wayland_current_resolution() -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    for output in outputs {
        if let Some(current_mode) = output.current_mode {
            println!("{}x{}", current_mode.width, current_mode.height);
        }
    }

    Ok(())
}

pub fn wayland_get_refresh_rate() -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    for output in outputs {
        if let Some(current_mode) = output.current_mode {
            println!("{}Hz", current_mode.refresh / 1000);
        }
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
