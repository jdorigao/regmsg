use clap::{Arg, Command, value_parser};
use std::env;
use std::process;

fn main() {
    // Check XDG session type
    let session_type = match env::var("XDG_SESSION_TYPE") {
        Ok(session_type) => session_type,
        Err(e) => {
            eprintln!("Error accessing XDG_SESSION_TYPE: {}", e);
            process::exit(1);
        }
    };

    let matches = Command::new("regmsg")
        .version("0.1.0")
        .author("REG-Linux")
        .about("Manages screen resolution and display settings")
        .subcommand(Command::new("listModes").about("Lists available display modes"))
        .subcommand(Command::new("listOutputs").about("Lists available outputs"))
        .subcommand(Command::new("currentMode").about("Shows the current display mode"))
        .subcommand(Command::new("currentOutput").about("Shows the current output"))
        .subcommand(Command::new("currentResolution").about("Shows the current resolution"))
        .subcommand(
            Command::new("setMode")
                .about("Sets the display mode")
                .arg(Arg::new("MODE").required(true)),
        )
        .subcommand(
            Command::new("setOutput")
                .about("Sets the output")
                .arg(Arg::new("OUTPUT").required(true)),
        )
        .subcommand(
            Command::new("setRotation")
                .about("Sets the rotation (0|1|2|3)")
                .arg(
                    Arg::new("ROTATION")
                        .required(true)
                        .value_parser(value_parser!(u8).range(0..=3)),
                ),
        )
        .subcommand(Command::new("getDisplayMode").about("Gets the current display mode"))
        .subcommand(Command::new("getRefreshRate").about("Gets the current refresh rate"))
        .get_matches();

    match matches.subcommand() {
        Some(("listModes", _)) => list_modes(),
        Some(("listOutputs", _)) => list_outputs(),
        Some(("currentMode", _)) => current_mode(),
        Some(("currentOutput", _)) => current_output(),
        Some(("currentResolution", _)) => current_resolution(),
        Some(("setMode", sub_matches)) => {
            let mode = sub_matches.get_one::<String>("MODE").unwrap();
            set_mode(mode);
        }
        Some(("setOutput", sub_matches)) => {
            let output = sub_matches.get_one::<String>("OUTPUT").unwrap();
            set_output(output);
        }
        Some(("setRotation", sub_matches)) => {
            let rotation = sub_matches.get_one::<u8>("ROTATION").unwrap();
            set_rotation(*rotation);
        }
        Some(("getDisplayMode", _)) => get_display_mode(&session_type),
        Some(("getRefreshRate", _)) => get_refresh_rate(),
        _ => {
            eprintln!("Invalid command. Use --help for usage information.");
            std::process::exit(1);
        }
    }
}

fn list_modes() {
    // Implement the logic to list display modes
    println!("Listing display modes...");
}

fn list_outputs() {
    // Implement the logic to list outputs
    println!("Listing outputs...");
}

fn current_mode() {
    // Implement the logic to show the current display mode
    println!("Showing current display mode...");
}

fn current_output() {
    // Implement the logic to show the current output
    println!("Showing current output...");
}

fn current_resolution() {
    // Implement the logic to show the current resolution
    println!("Showing current resolution...");
}

fn set_mode(mode: &str) {
    // Implement the logic to set the display mode
    println!("Setting display mode to {}...", mode);
}

fn set_output(output: &str) {
    // Implement the logic to set the output
    println!("Setting output to {}...", output);
}

fn set_rotation(rotation: u8) {
    // Implement the logic to set the rotation
    println!("Setting rotation to {}...", rotation);
}

fn get_display_mode(session_type: &str) {
    match session_type {
        "x11" => println!("Using backend X11."),
        "wayland" => println!("Using backend Wayland."),
        _ => println!("Using backend KMS/DRM."),
    }
}

fn get_refresh_rate() {
    // Implement the logic to get the current refresh rate
    println!("Getting current refresh rate...");
}
