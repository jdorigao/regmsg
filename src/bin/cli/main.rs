#![cfg(feature = "cli")]

use clap::{Parser, Subcommand};
use log::{debug, error};
use zeromq::ReqSocket; // or DealerSocket, RouterSocket, etc.
use zeromq::ZmqMessage;
use zeromq::prelude::*; // traits

/// Global CLI arguments
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Target screen identifier (optional)
    #[arg(short = 's', long)]
    screen: Option<String>,

    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,

    /// Additional arguments passed to the daemon
    #[arg(last = true)]
    args: Vec<String>,
}

/// List of available subcommands
#[derive(Subcommand, Debug)]
#[command(rename_all = "camelCase")] // <--- all variants become camelCase
enum Commands {
    #[command(about = "Lists all available outputs (e.g., HDMI, VGA).")]
    ListModes,

    #[command(about = "List all available display outputs")]
    ListOutputs,
    #[command(about = "Displays the current display mode for the specified screen.")]
    CurrentMode,
    #[command(about = "Displays the current output (e.g., HDMI, VGA).")]
    CurrentOutput,
    #[command(about = "Displays the current resolution for the specified screen.")]
    CurrentResolution,
    #[command(about = "Displays the current screen rotation for the specified screen.")]
    CurrentRotation,
    #[command(about = "Displays the current refresh rate for the specified screen.")]
    CurrentRefresh,
    #[command(about = "Displays the current window system.")]
    CurrentBackend,
    #[command(about = "Sets the display mode for the specified screen.")]
    SetMode { mode: String },
    #[command(about = "Sets the output resolution and refresh rate (e.g., WxH@R or WxH).")]
    SetOutput { output: String },
    #[command(about = "Sets the screen rotation for the specified screen.")]
    SetRotation {
        #[arg(value_parser = ["0", "90", "180", "270"])]
        rotation: String,
    },
    #[command(about = "Takes a screenshot of the current screen.")]
    GetScreenshot,
    #[command(about = "Maps the touchscreen to the correct display.")]
    MapTouchScreen,
    #[command(
        about = "Sets the screen resolution to the maximum supported resolution (e.g., 1920x1080)."
    )]
    MinToMaxResolution,
}

/// Entry point
#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger with flexible configuration via environment variables
    env_logger::init();

    let cli = Cli::parse();

    // Connect to the daemon via ZeroMQ
    let mut socket = ReqSocket::new();
    let _ = socket.connect("ipc:///var/run/regmsgd.sock").await;

    // Execute the command
    if let Err(e) = handle_command(&cli, socket).await {
        error!("Error: {e}");
        std::process::exit(1);
    }

    Ok(())
}

/// Execute the selected subcommand
async fn handle_command(
    cli: &Cli,
    mut socket: zeromq::ReqSocket,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut msg = String::new();

    // Build the command based on the enum
    match &cli.command {
        Commands::ListModes => msg.push_str("listModes"),
        Commands::ListOutputs => msg.push_str("listOutputs"),
        Commands::CurrentMode => msg.push_str("currentMode"),
        Commands::CurrentOutput => msg.push_str("currentOutput"),
        Commands::CurrentResolution => msg.push_str("currentResolution"),
        Commands::CurrentRotation => msg.push_str("currentRotation"),
        Commands::CurrentRefresh => msg.push_str("currentRefresh"),
        Commands::CurrentBackend => msg.push_str("currentBackend"),
        Commands::SetMode { mode } => {
            msg.push_str("setMode ");
            msg.push_str(mode);
        }
        Commands::SetOutput { output } => {
            msg.push_str("setOutput ");
            msg.push_str(output);
        }
        Commands::SetRotation { rotation } => {
            msg.push_str("setRotation ");
            msg.push_str(rotation);
        }
        Commands::GetScreenshot => msg.push_str("getScreenshot"),
        Commands::MapTouchScreen => msg.push_str("mapTouchScreen"),
        Commands::MinToMaxResolution => msg.push_str("minToMaxResolution"),
    }

    // Add --screen if specified
    if let Some(screen) = &cli.screen {
        msg.push_str(" --screen ");
        msg.push_str(screen);
    }

    // Add additional arguments
    if !cli.args.is_empty() {
        msg.push(' ');
        msg.push_str(&cli.args.join(" "));
    }

    // Send command to the daemon
    let _ = socket.send(ZmqMessage::from(msg.clone())).await;

    // Receive and display response
    let reply = socket.recv().await?;

    // Get the first frame as a UTF-8 string
    let reply_str = match reply.get(0) {
        Some(frame) => String::from_utf8(frame.to_vec())?,
        None => String::new(),
    };

    debug!("{}", reply_str); // prints the raw string

    Ok(())
}
