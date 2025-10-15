#![cfg(feature = "cli")]

use clap::{Parser, Subcommand};
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger};
use std::fs::OpenOptions;

use zeromq::ReqSocket; // or DealerSocket, RouterSocket, etc.
use zeromq::ZmqMessage;
use zeromq::prelude::*; // traits

/// Arguments globaux de la CLI
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Identifiant de l'écran cible (optionnel)
    #[arg(short, long)]
    screen: Option<String>,

    /// Active le logging terminal
    #[arg(short, long)]
    log: bool,

    /// Sous-commande à exécuter
    #[command(subcommand)]
    command: Commands,

    /// Arguments supplémentaires transmis au démon
    #[arg(last = true)]
    args: Vec<String>,
}

/// Liste des sous-commandes disponibles
#[derive(Subcommand, Debug)]
#[command(rename_all = "camelCase")] // <--- all variants become camelCase
enum Commands {
    ListModes,
    ListOutputs,
    CurrentMode,
    CurrentOutput,
    CurrentResolution,
    CurrentRotation,
    CurrentRefresh,
    CurrentBackend,
    SetMode {
        mode: String,
    },
    SetOutput {
        output: String,
    },
    SetRotation {
        #[arg(value_parser = ["0", "90", "180", "270"])]
        rotation: String,
    },
    GetScreenshot,
    MapTouchScreen,
    MinToMaxResolution,
}

/// Configure le logging fichier + terminal
fn init_logging(enable_terminal: bool) {
    let mut loggers: Vec<Box<dyn simplelog::SharedLogger>> = vec![WriteLogger::new(
        LevelFilter::Debug,
        Config::default(),
        OpenOptions::new()
            .create(true)
            .append(true)
            .open("/var/log/regmsg.log")
            .expect("Impossible d'ouvrir /var/log/regmsg.log"),
    )];

    if enable_terminal {
        loggers.push(TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        ));
    }

    CombinedLogger::init(loggers).expect("Impossible d'initialiser le logger");
}

/// Point d’entrée
#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Init logging
    init_logging(cli.log);

    // Connexion au démon via ZeroMQ
    //let ctx = zmq::Context::new();
    let mut socket = ReqSocket::new();
    let _ = socket.connect("ipc:///var/run/regmsgd.sock").await;

    // Exécution de la commande
    if let Err(e) = handle_command(&cli, socket).await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    Ok(())
}

/// Exécute la sous-commande sélectionnée
async fn handle_command(
    cli: &Cli,
    mut socket: zeromq::ReqSocket,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut msg = String::new();

    // Construire la commande en fonction de l'enum
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

    // Ajouter --screen si précisé
    if let Some(screen) = &cli.screen {
        msg.push_str(" --screen ");
        msg.push_str(screen);
    }

    // Ajouter arguments supplémentaires
    if !cli.args.is_empty() {
        msg.push(' ');
        msg.push_str(&cli.args.join(" "));
    }

    // Envoyer la commande au démon
    let _ = socket.send(ZmqMessage::from(msg.clone())).await;

    // Réception et affichage
    let reply = socket.recv().await?;

    // Get the first frame as a UTF-8 string
    let reply_str = match reply.get(0) {
        Some(frame) => String::from_utf8(frame.to_vec())?,
        None => String::new(),
    };

    println!("{}", reply_str); // prints the raw string

    //let reply_str = String::from_utf8(reply[0].to_vec())?;

    /*match reply {
        Ok(text) => println!("{:?}", text),
        Err(bytes) => println!("Error in reply (non-UTF8): {:?}", bytes),
    }*/

    Ok(())
}
