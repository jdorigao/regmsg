#![cfg(feature = "daemon")]

use zmq::{Context, REP};
use std::fs;

mod screen;

fn main() {
    let socket_path = "/var/run/regmsgd.sock";

    // Supprimer le socket existant si présent
    let _ = fs::remove_file(socket_path);

    // Créer le contexte ZMQ
    let context = Context::new();

    // Créer un socket REP (reply)
    let socket = context.socket(REP).expect("Failed to create socket");
    socket
        .bind(&format!("ipc://{}", socket_path))
        .expect("Failed to bind socket");
    println!("Daemon running on ipc://{} ...", socket_path);

    loop {
        // Recevoir le message
        let msg = socket.recv_string(0).expect("Failed to receive message");

        match msg {
            Ok(cmdline) => {
                println!("Received: {}", cmdline);

                // Découper la commande et ses arguments
                let parts: Vec<&str> = cmdline.split_whitespace().collect();
                let (cmd, args) = parts.split_first().unwrap_or((&"", &[]));

                // Exécuter la commande correspondante
                let result: Result<String, Box<dyn std::error::Error>> = match *cmd {
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
                    "getScreenshot" => screen::get_screenshot()
                        .map(|_| "Screenshot taken".to_string()),
                    "mapTouchScreen" => screen::map_touch_screen()
                        .map(|_| "Touchscreen mapped".to_string()),
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
                        cmdline
                    ))),
                };

                // Convertir le résultat en chaîne pour l'envoyer
                let reply = match result {
                    Ok(msg) => msg,
                    Err(err) => format!("Error: {}", err),
                };

                // Envoyer la réponse au client
                socket.send(&reply, 0).expect("Failed to send reply");
            }
            Err(e) => eprintln!("Error receiving message: {:?}", e),
        }
    }
}
