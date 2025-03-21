use clap::{Arg, Command};

mod screen;

fn main() {
    let matches = Command::new("regmsg")
        .version("0.0.0")
        .author("REG-Linux")
        .about("Manages screen resolution and display settings")
        .arg(
            Arg::new("screen")
                .short('s')
                .long("screen")
                .help("Specifies the screen to apply the command to")
                .global(true),
        )
        .subcommand(Command::new("listModes").about("Lists available display modes"))
        .subcommand(Command::new("listOutputs").about("Lists available outputs"))
        .subcommand(Command::new("currentMode").about("Shows the current display mode"))
        .subcommand(Command::new("currentOutput").about("Shows the current output"))
        .subcommand(Command::new("currentResolution").about("Shows the current resolution"))
        .subcommand(Command::new("currentRefresh").about("Shows the current refresh rate"))
        .subcommand(
            Command::new("setMode")
                .about("Sets the display mode")
                .arg(Arg::new("MODE").required(true)),
        )
        .subcommand(
            Command::new("setOutput")
                .about("Sets the output WxH@R or WxH")
                .arg(Arg::new("OUTPUT").required(true)),
        )
        .subcommand(
            Command::new("setRotation")
                .about("Sets the rotation 0|90|180|270")
                .arg(
                    Arg::new("ROTATION")
                        .required(true)
                        .value_parser(["0", "90", "180", "270"]),
                ),
        )
        .subcommand(Command::new("getScreenshot").about("Get screenshot"))
        .subcommand(Command::new("mapTouchScreen").about("Maps the touchscreen"))
        .subcommand(Command::new("minTomaxResolution").about("Maximum resolution 1920x1080"))
        .get_matches();

    // Obter o valor do argumento `screen` do nível superior
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
