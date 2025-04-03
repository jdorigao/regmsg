use clap::{Arg, Command}; // Importing command-line argument parsing utilities
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger}; // Importing logging utilities
use std::fs::OpenOptions; // For file handling

mod screen; // Importing the screen module

fn main() {
    // Configure and parse command-line arguments
    let matches = Command::new("regmsg")
        .version("0.0.1") // Application version
        .author("Juliano Dorigão") // Author information
        .about("A tool to manage screen resolution and display settings.") // Short description
        .arg(
            Arg::new("screen") // Argument for specifying the screen
                .short('s')
                .long("screen")
                .help("Specifies the screen to apply the command to. Use this option to target a specific screen.")
                .global(true), // Available for all subcommands
        )
        .arg(
            Arg::new("log") // Enable logging flag
                .short('l')
                .long("log")
                .help("Enables detailed logging to the terminal")
                .action(clap::ArgAction::SetTrue), // Sets flag to true when used
        )
        // Subcommands for different functionalities
        .subcommand(Command::new("listModes").about("Lists all available display modes for the specified screen."))
        .subcommand(Command::new("listOutputs").about("Lists all available outputs (e.g., HDMI, VGA)."))
        .subcommand(Command::new("currentMode").about("Displays the current display mode for the specified screen."))
        .subcommand(Command::new("currentOutput").about("Displays the current output (e.g., HDMI, VGA)."))
        .subcommand(Command::new("currentResolution").about("Displays the current resolution for the specified screen."))
        .subcommand(Command::new("currentRefresh").about("Displays the current refresh rate for the specified screen."))
        .subcommand(Command::new("currentBackend").about("Displays the current window system."))
        .subcommand(
            Command::new("setMode") // Set a specific display mode
                .about("Sets the display mode for the specified screen.")
                .arg(Arg::new("MODE").required(true).help("The display mode to set (e.g., 1920x1080@60).")),
        )
        .subcommand(
            Command::new("setOutput") // Set output resolution and refresh rate
                .about("Sets the output resolution and refresh rate (e.g., WxH@R or WxH).")
                .arg(Arg::new("OUTPUT").required(true).help("The output resolution and refresh rate to set (e.g., 1920x1080@60).")),
        )
        .subcommand(
            Command::new("setRotation") // Set screen rotation
                .about("Sets the screen rotation for the specified screen.")
                .arg(
                    Arg::new("ROTATION")
                        .required(true)
                        .value_parser(["0", "90", "180", "270"]) // Only allow valid angles
                        .help("The rotation angle to set (0, 90, 180, or 270 degrees)."),
                ),
        )
        .subcommand(Command::new("getScreenshot").about("Takes a screenshot of the current screen."))
        .subcommand(Command::new("mapTouchScreen").about("Maps the touchscreen to the correct display."))
        .subcommand(Command::new("minTomaxResolution").about("Sets the screen resolution to the maximum supported resolution (e.g., 1920x1080)."))
        .get_matches(); // Parse arguments

    // Set up logging
    let mut loggers: Vec<Box<dyn simplelog::SharedLogger>> = vec![
        // File logger: logs all debug-level messages to /var/log/regmsg.log
        WriteLogger::new(
            LevelFilter::Debug, // Log all debug-level messages
            Config::default(),
            OpenOptions::new()
                .create(true) // Create file if it doesn't exist
                .append(true) // Append instead of overwriting
                .open("/var/log/regmsg.log")
                .expect("Failed to open log file in append mode"), // Handle failure
        ),
    ];

    // Enable terminal logging if --log flag is set
    if matches.get_flag("log") {
        loggers.push(TermLogger::new(
            LevelFilter::Info, // Log info-level messages
            Config::default(),
            TerminalMode::Mixed, // Output logs to both stdout and stderr
            simplelog::ColorChoice::Auto, // Use color output if terminal supports it
        ));
    }

    // Initialize the combined logger
    CombinedLogger::init(loggers).expect("Failed to initialize logger");

    // Get screen argument if provided
    let screen = matches.get_one::<String>("screen").map(|s| s.as_str());

    // Match and execute the selected subcommand
    if let Err(e) = match matches.subcommand() {
        Some(("listModes", _)) => screen::list_modes(screen), // List available display modes
        Some(("listOutputs", _)) => screen::list_outputs(), // List available outputs
        Some(("currentMode", _)) => screen::current_mode(screen), // Show current display mode
        Some(("currentOutput", _)) => screen::current_output(), // Show current output
        Some(("currentResolution", _)) => screen::current_resolution(screen), // Show current resolution
        Some(("currentRefresh", _)) => screen::current_refresh(screen), // Show current refresh rate
        Some(("currentBackend", _)) => screen::current_backend(), // Show current window system
        Some(("setMode", sub_matches)) => {
            let mode = sub_matches.get_one::<String>("MODE").unwrap(); // Retrieve mode argument
            screen::set_mode(screen, mode).map(|_| "Mode set successfully".to_string()) // Set the mode
        }
        Some(("setOutput", sub_matches)) => {
            let output = sub_matches.get_one::<String>("OUTPUT").unwrap(); // Retrieve output argument
            screen::set_output(output).map(|_| "Output set successfully".to_string()) // Set the output
        }
        Some(("setRotation", sub_matches)) => {
            let rotation = sub_matches.get_one::<String>("ROTATION").unwrap(); // Retrieve rotation argument
            screen::set_rotation(screen, rotation).map(|_| "Rotation set successfully".to_string()) // Set rotation
        }
        Some(("getScreenshot", _)) => {
            screen::get_screenshot().map(|_| "Screenshot taken successfully".to_string()) // Capture a screenshot
        }
        Some(("mapTouchScreen", _)) => {
            screen::map_touch_screen().map(|_| "Touch screen successfully mapped".to_string()) // Map touchscreen
        }
        Some(("minTomaxResolution", _)) => screen::min_to_max_resolution(screen)
            .map(|_| "Resolution set to maximum successfully".to_string()), // Set max resolution
        _ => {
            // Handle unknown subcommands
            eprintln!("Invalid command. Use --help for usage information.");
            std::process::exit(1); // Exit with error code
        }
    } {
        // Handle errors from function execution
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
