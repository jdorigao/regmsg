#![cfg(feature = "daemon")]

use std::fs;
use zeromq::prelude::*; // traits
use zeromq::{RepSocket, ZmqMessage};

mod screen;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = "/var/run/regmsgd.sock";

    // Supprimer le socket existant si présent
    let _ = fs::remove_file(socket_path);

    // Créer un socket REP (reply)
    let mut socket = RepSocket::new();
    socket.bind(&format!("ipc://{}", socket_path)).await?;
    println!("Daemon running on ipc://{} ...", socket_path);

    loop {
        let msg = socket.recv().await;

        match msg {
            Ok(cmdline) => {
                // Convert first frame to String
                let cmdline_str = match cmdline.get(0) {
                    Some(frame) => match String::from_utf8(frame.to_vec()) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Invalid UTF-8 message: {:?}", e);
                            continue;
                        }
                    },
                    None => {
                        eprintln!("Received empty message");
                        continue;
                    }
                };

                // Split command and arguments
                let parts: Vec<&str> = cmdline_str.split_whitespace().collect();
                let (cmd, args) = parts.split_first().unwrap_or((&"", &[]));

                // Execute command and send reply...
                let result: Result<String, Box<dyn std::error::Error>> =
                    match *cmd {
                        // Fonctions qui renvoient Result<String, Box<dyn Error>>
                        "listModes" => screen::list_modes(None),
                        "listOutputs" => screen::list_outputs(),
                        "currentMode" => screen::current_mode(None),
                        "currentOutput" => screen::current_output(),
                        "currentResolution" => screen::current_resolution(None),
                        "currentRotation" => screen::current_rotation(None),
                        "currentRefresh" => screen::current_refresh(None),
                        "currentBackend" => screen::current_backend(),

                        // Fonctions qui renvoient Result<(), Box<dyn Error>>
                        "getScreenshot" => {
                            screen::get_screenshot().map(|_| "Screenshot taken".to_string())
                        }
                        "mapTouchScreen" => {
                            screen::map_touch_screen().map(|_| "Touchscreen mapped".to_string())
                        }
                        "minTomaxResolution" => screen::min_to_max_resolution(None)
                            .map(|_| "Resolution set to max".to_string()),

                        // Commandes avec arguments
                        "setMode" if args.len() == 1 => screen::set_mode(None, args[0])
                            .map(|_| format!("Mode set to {}", args[0])),
                        "setOutput" if args.len() == 1 => screen::set_output(args[0])
                            .map(|_| format!("Output set to {}", args[0])),
                        "setRotation" if args.len() == 1 => screen::set_rotation(None, args[0])
                            .map(|_| format!("Rotation set to {}", args[0])),

                        // Commande vide
                        "" => Err(Box::<dyn std::error::Error>::from("Empty command")),

                        // Commande inconnue
                        _ => Err(Box::<dyn std::error::Error>::from(format!(
                            "Unknown or invalid command: {}",
                            cmdline_str
                        ))),
                    };

                // Convertir le résultat en chaîne pour l'envoyer
                let reply = match result {
                    Ok(msg) => msg,
                    Err(err) => format!("Error: {}", err),
                };

                // Envoyer la réponse au client
                if let Err(e) = socket.send(ZmqMessage::from(reply)).await {
                    eprintln!("Failed to send reply: {:?}", e);
                }
            }
            Err(e) => eprintln!("Error receiving message: {:?}", e),
        }
    }
}
