use clap::{Arg, Command};

mod screen;

fn main() {
    env_logger::init();

    let matches = Command::new("regmsg")
        .version("0.0.0")
        .author("REG-Linux")
        .about("A tool to manage screen resolution and display settings.")
        .arg(
            Arg::new("screen")
                .short('s')
                .long("screen")
                .help("Specifies the screen to apply the command to. Use this option to target a specific screen.")
                .global(true),
        )
        .subcommand(Command::new("listModes").about("Lists all available display modes for the specified screen."))
        .subcommand(Command::new("listOutputs").about("Lists all available outputs (e.g., HDMI, VGA)."))
        .subcommand(Command::new("currentMode").about("Displays the current display mode for the specified screen."))
        .subcommand(Command::new("currentOutput").about("Displays the current output (e.g., HDMI, VGA)."))
        .subcommand(Command::new("currentResolution").about("Displays the current resolution for the specified screen."))
        .subcommand(Command::new("currentRefresh").about("Displays the current refresh rate for the specified screen."))
        .subcommand(
            Command::new("setMode")
                .about("Sets the display mode for the specified screen.")
                .arg(Arg::new("MODE").required(true).help("The display mode to set (e.g., 1920x1080@60).")),
        )
        .subcommand(
            Command::new("setOutput")
                .about("Sets the output resolution and refresh rate (e.g., WxH@R or WxH).")
                .arg(Arg::new("OUTPUT").required(true).help("The output resolution and refresh rate to set (e.g., 1920x1080@60).")),
        )
        .subcommand(
            Command::new("setRotation")
                .about("Sets the screen rotation for the specified screen.")
                .arg(
                    Arg::new("ROTATION")
                        .required(true)
                        .value_parser(["0", "90", "180", "270"])
                        .help("The rotation angle to set (0, 90, 180, or 270 degrees)."),
                ),
        )
        .subcommand(Command::new("getScreenshot").about("Takes a screenshot of the current screen."))
        .subcommand(Command::new("mapTouchScreen").about("Maps the touchscreen to the correct display."))
        .subcommand(Command::new("minTomaxResolution").about("Sets the screen resolution to the maximum supported resolution (e.g., 1920x1080)."))
        .get_matches();

    // Get the value of the top-level `screen` argument
    let screen = matches.get_one::<String>("screen").map(|s| s.as_str());

    if let Err(e) = match matches.subcommand() {
        Some(("listModes", _)) => screen::list_modes(screen),
        Some(("listOutputs", _)) => screen::list_outputs(),
        Some(("currentMode", _)) => screen::current_mode(screen),
        Some(("currentOutput", _)) => screen::current_output(),
        Some(("currentResolution", _)) => screen::current_resolution(screen),
        Some(("currentRefresh", _)) => screen::current_refresh(screen),
        Some(("setMode", sub_matches)) => {
            let mode = sub_matches.get_one::<String>("MODE").unwrap();
            screen::set_mode(screen, mode).map(|_| "Mode set successfully".to_string())
        }
        Some(("setOutput", sub_matches)) => {
            let output = sub_matches.get_one::<String>("OUTPUT").unwrap();
            screen::set_output(output).map(|_| "Output set successfully".to_string())
        }
        Some(("setRotation", sub_matches)) => {
            let rotation = sub_matches.get_one::<String>("ROTATION").unwrap();
            screen::set_rotation(screen, rotation).map(|_| "Rotation set successfully".to_string())
        }
        Some(("getScreenshot", _)) => screen::get_screenshot().map(|_| "Screenshot taken successfully".to_string()),
        Some(("mapTouchScreen", _)) => screen::map_touch_screen().map(|_| "Touch screen successfully mapped".to_string()),
        Some(("minTomaxResolution", _)) => screen::min_to_max_resolution(screen).map(|_| "Resolution set to maximum successfully".to_string()),
        _ => {
            eprintln!("Invalid command. Use --help for usage information.");
            std::process::exit(1);
        }
    } {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
