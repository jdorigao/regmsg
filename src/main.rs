use clap::{Arg, Command};

mod screen;

fn main() {
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
        .subcommand(Command::new("getDisplayMode").about("Gets the current display mode"))
        .subcommand(Command::new("getRefreshRate").about("Gets the current refresh rate"))
        .subcommand(Command::new("getScreenshot").about("Get screenshot"))
        .get_matches();

    match matches.subcommand() {
        Some(("listModes", _)) => screen::list_modes(),
        Some(("listOutputs", _)) => screen::list_outputs(),
        Some(("currentMode", _)) => screen::current_mode(),
        Some(("currentOutput", _)) => screen::current_output(),
        Some(("currentResolution", _)) => screen::current_resolution(),
        Some(("setMode", sub_matches)) => {
            let mode = sub_matches.get_one::<String>("MODE").unwrap();
            if let Err(e) = screen::set_mode(mode) {
                eprintln!("Error setting mode: {}", e);
                std::process::exit(1);
            }
        }
        Some(("setOutput", sub_matches)) => {
            let output = sub_matches.get_one::<String>("OUTPUT").unwrap();
            screen::set_output(output);
        }
        Some(("setRotation", sub_matches)) => {
            let rotation = sub_matches.get_one::<String>("ROTATION").unwrap();
            screen::set_rotation(rotation);
        }
        Some(("getDisplayMode", _)) => screen::get_display_mode(),
        Some(("getRefreshRate", _)) => screen::get_refresh_rate(),
        Some(("getScreenshot", _)) => screen::get_screenshot(),
        _ => {
            eprintln!("Invalid command. Use --help for usage information.");
            std::process::exit(1);
        }
    }
}
