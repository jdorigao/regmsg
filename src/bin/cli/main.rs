#![cfg(feature = "cli")]

use clap::{Parser, Subcommand};
use tracing::{error, info, debug};
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
    // Initialize tracing with console output only for CLI
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    debug!("Parsed CLI arguments: {:?}", cli);

    // Connect to the daemon via ZeroMQ
    let mut socket = ReqSocket::new();
    match socket.connect("ipc:///var/run/regmsgd.sock").await {
        Ok(_) => debug!("Successfully connected to regmsg daemon"),
        Err(e) => {
            error!("Failed to connect to daemon: {e}");
            return Err(Box::new(e));
        }
    }

    // Execute the command
    if let Err(e) = handle_command(&cli, socket).await {
        error!("Error executing command: {e}");
        std::process::exit(1);
    }

    info!("Command executed successfully");
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
        Commands::ListModes => {
            msg.push_str("listModes");
            info!("Listing available display modes");
        },
        Commands::ListOutputs => {
            msg.push_str("listOutputs");
            info!("Listing available display outputs");
        },
        Commands::CurrentMode => {
            msg.push_str("currentMode");
            info!("Getting current display mode");
        },
        Commands::CurrentOutput => {
            msg.push_str("currentOutput");
            info!("Getting current display output");
        },
        Commands::CurrentResolution => {
            msg.push_str("currentResolution");
            info!("Getting current display resolution");
        },
        Commands::CurrentRotation => {
            msg.push_str("currentRotation");
            info!("Getting current screen rotation");
        },
        Commands::CurrentRefresh => {
            msg.push_str("currentRefresh");
            info!("Getting current refresh rate");
        },
        Commands::CurrentBackend => {
            msg.push_str("currentBackend");
            info!("Getting current window backend");
        },
        Commands::SetMode { mode } => {
            msg.push_str("setMode ");
            msg.push_str(mode);
            info!("Setting display mode to: {}", mode);
        },
        Commands::SetOutput { output } => {
            msg.push_str("setOutput ");
            msg.push_str(output);
            info!("Setting output to: {}", output);
        },
        Commands::SetRotation { rotation } => {
            msg.push_str("setRotation ");
            msg.push_str(rotation);
            info!("Setting screen rotation to: {} degrees", rotation);
        },
        Commands::GetScreenshot => {
            msg.push_str("getScreenshot");
            info!("Taking screenshot");
        },
        Commands::MapTouchScreen => {
            msg.push_str("mapTouchScreen");
            info!("Mapping touchscreen to display");
        },
        Commands::MinToMaxResolution => {
            msg.push_str("minToMaxResolution");
            info!("Setting resolution to maximum supported");
        },
    }

    // Add --screen if specified
    if let Some(screen) = &cli.screen {
        msg.push_str(" --screen ");
        msg.push_str(screen);
        debug!("Using screen: {}", screen);
    }

    // Add additional arguments
    if !cli.args.is_empty() {
        msg.push(' ');
        msg.push_str(&cli.args.join(" "));
        debug!("Additional arguments: {:?}", cli.args);
    }

    debug!("Sending command to daemon: {}", msg);

    // Send command to the daemon
    match socket.send(ZmqMessage::from(msg.clone())).await {
        Ok(_) => debug!("Command sent successfully"),
        Err(e) => error!("Failed to send command: {}", e),
    }

    // Receive and display response
    debug!("Waiting for response from daemon...");
    let reply = socket.recv().await?;
    debug!("Received response from daemon");

    // Get the first frame as a UTF-8 string
    let reply_str = match reply.get(0) {
        Some(frame) => String::from_utf8(frame.to_vec())?,
        None => String::new(),
    };

    if !reply_str.is_empty() {
        // Log non-empty responses at info level, empty at debug level
        if reply_str.trim().is_empty() {
            debug!("Received empty response from daemon");
        } else {
            info!("Daemon response: {}", reply_str.trim());
        }
    } else {
        debug!("Received empty response from daemon");
    }

    println!("{}", reply_str); // print the result to stdout

    Ok(())
}
