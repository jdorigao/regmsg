use log::debug;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::sync::{Arc, Mutex, OnceLock};

pub type Guid = String;

#[derive(Debug, Clone)]
pub struct ControllerMapping {
    pub mapping_data: String,
}

static CONTROLLER_DB: OnceLock<HashMap<Guid, ControllerMapping>> = OnceLock::new();
static SDL_CONTROLLER_DB: OnceLock<Arc<Mutex<HashMap<Guid, ControllerMapping>>>> = OnceLock::new();

fn load_gamecontroller_db() -> io::Result<HashMap<Guid, ControllerMapping>> {
    let paths = [
        "/userdata/system/configs/emulationstation/gamecontrollerdb.txt",
        "/usr/share/emulationstation/gamecontrollerdb.txt",
    ];

    for path in &paths {
        match fs::File::open(path) {
            Ok(file) => {
                debug!("Loading gamecontrollerdb from {}", path);
                let reader = BufReader::new(file);
                let mut map = HashMap::new();

                for line in reader.lines() {
                    let line = line?;
                    if line.trim().is_empty() || line.starts_with('#') {
                        continue;
                    }

                    // Cada entrada tem o formato: GUID,nome,botões...,platform:...
                    // O GUID é sempre o primeiro campo antes da primeira vírgula
                    if let Some(comma) = line.find(',') {
                        let guid = &line[..comma];
                        // Armazena a linha inteira ou só o mapeamento (sem GUID)
                        let controller_mapping = ControllerMapping {
                            mapping_data: line[comma + 1..].to_string(),
                        };
                        map.insert(guid.to_string(), controller_mapping);
                    }
                }
                // Successfully loaded from this path, return the map
                return Ok(map);
            }
            Err(_) => {
                // If this path failed, continue to try the next one
                continue;
            }
        }
    }

    // If no paths were successful, return an error
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("None of the specified paths could be opened: {:?}", paths),
    ))
}

fn get_controller_db()
-> Result<&'static HashMap<Guid, ControllerMapping>, Box<dyn std::error::Error>> {
    // Check if the controller database is already initialized
    if CONTROLLER_DB.get().is_some() {
        debug!("Controller database already initialized");
    } else {
        debug!("Initializing controller database");
        let db = load_gamecontroller_db()?;
        CONTROLLER_DB.set(db).map_err(|_| {
            io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Controller database already initialized",
            )
        })?;
    }

    CONTROLLER_DB.get().ok_or_else(|| {
        Box::<dyn std::error::Error>::from(io::Error::new(
            io::ErrorKind::NotFound,
            "Controller database not loaded",
        ))
    })
}

pub fn add_controllers_to_sdl_db(guids: Vec<Guid>) -> Result<(), Box<dyn std::error::Error>> {
    let controller_db = get_controller_db()?;

    // Initialize SDL_CONTROLLER_DB if not already done
    if SDL_CONTROLLER_DB.get().is_none() {
        let sdl_db = Arc::new(Mutex::new(HashMap::new()));
        SDL_CONTROLLER_DB.set(sdl_db).map_err(|_| {
            Box::<dyn std::error::Error>::from(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "SDL controller database already initialized",
            ))
        })?;
    }

    // Get access to the SDL controller database and add the requested GUIDs
    if let Some(sdl_db_mutex) = SDL_CONTROLLER_DB.get() {
        let mut sdl_db = sdl_db_mutex.lock().map_err(|e| {
            Box::<dyn std::error::Error>::from(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to lock SDL controller database: {}", e),
            ))
        })?;

        for guid in guids {
            if let Some(mapping) = controller_db.get(&guid) {
                sdl_db.insert(guid, mapping.clone());
            }
        }
    }

    Ok(())
}

pub fn get_sdl_controller_db()
-> Result<Arc<Mutex<HashMap<Guid, ControllerMapping>>>, Box<dyn std::error::Error>> {
    if SDL_CONTROLLER_DB.get().is_none() {
        let sdl_db = Arc::new(Mutex::new(HashMap::new()));
        SDL_CONTROLLER_DB.set(sdl_db).map_err(|_| {
            Box::<dyn std::error::Error>::from(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "SDL controller database already initialized",
            ))
        })?;
    }

    SDL_CONTROLLER_DB.get().cloned().ok_or_else(|| {
        Box::<dyn std::error::Error>::from(io::Error::new(
            io::ErrorKind::NotFound,
            "SDL controller database not available",
        ))
    })
}
