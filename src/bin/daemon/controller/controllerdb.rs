use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::sync::{Arc, Mutex, OnceLock};
use tracing::debug;

use crate::config;

/// Represents a controller with its GUID, name and input mappings
#[derive(Debug, Clone, serde::Serialize)]
pub struct Controller {
    /// The GUID of the controller
    pub guid: String,
    /// The name of the controller
    pub name: String,
    /// The input mappings as a HashMap
    pub inputs: HashMap<String, String>,
}

/// Static variable to store SDL controller configurations using Controller struct
///
/// This variable holds controller configurations in a thread-safe manner,
/// using the Controller struct which contains GUID, name and input mappings.
/// Uses HashMap for efficient insertions and removals by index.
static SDL_CONTROLLER_CONFIG: OnceLock<Arc<Mutex<HashMap<usize, Controller>>>> = OnceLock::new();

/// Parses the controller mapping data into a name and input mappings
///
/// This function takes the mapping_data string and extracts the controller name
/// and creates a HashMap of input mappings from it.
///
/// # Arguments
/// * `mapping_data` - The mapping data string in format "ControllerName,a:b0,b:b1,..."
///
/// # Returns
/// A tuple containing the controller name and a HashMap of input mappings
fn parse_controller_mapping_data(mapping_data: &str) -> (String, HashMap<String, String>) {
    let parts: Vec<&str> = mapping_data.split(',').collect();

    if parts.is_empty() {
        return ("Unknown".to_string(), HashMap::new());
    }

    // The first part is the controller name
    let controller_name = parts[0].to_string();

    // Process the remaining parts to build input mappings
    let mut inputs = HashMap::new();

    for part in &parts[1..] {
        if let Some(pos) = part.find(':') {
            let key = part[..pos].to_string();
            let value = part[pos + 1..].to_string();
            inputs.insert(key, value);
        }
    }

    (controller_name, inputs)
}

/// Finds a controller mapping in the game controller database files
///
/// This function searches for a controller mapping with the specified GUID in the
/// game controller database files. It checks both the user data location and the
/// system location, returning the mapping data if found.
///
/// # Arguments
/// * `guid_to_find` - The GUID to search for in the database files
///
/// # Returns
/// * `Ok(Some(String))` - If the GUID was found in the database, with the mapping data
/// * `Ok(None)` - If the GUID was not found in any database file
/// * `Err(io::Error)` - If there was an error reading the database files
///
/// # Format
/// The function expects lines in the format: `GUID,ControllerName,button_mappings,platform:Platform`
pub fn find_gamecontroller_db(guid_to_find: &str) -> io::Result<Option<String>> {
    for path in config::GAMECONTROLLER_DB_PATHS {
        // Check if the file exists before attempting to open it
        if !std::path::Path::new(path).exists() {
            debug!("File does not exist: {}", path);
            continue;
        }

        match fs::File::open(path) {
            Ok(file) => {
                debug!(
                    "Loading gamecontrollerdb from {} to find GUID {}",
                    path, guid_to_find
                );
                let reader = BufReader::new(file);

                for line in reader.lines() {
                    let line = line?;
                    if line.trim().is_empty() || line.starts_with('#') {
                        continue;
                    }

                    // Each entry has the format: GUID,name,buttons...,platform:...
                    // The GUID is always the first field before the first comma
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 3 {
                        let guid = parts[0];
                        if guid == guid_to_find {
                            // Found the GUID we're looking for, return the mapping data
                            // taking everything after the GUID (name, buttons, platform)
                            let mapping_data = parts[1..].join(",");
                            return Ok(Some(mapping_data));
                        }
                    }
                }
                // GUID not found in this file, continue to the next file
            }
            Err(e) => {
                debug!("Error opening file {}: {}", path, e);
                continue;
            }
        }
    }

    // If none of the files contained the requested GUID, return Ok(None)
    Ok(None)
}

/// Adds SDL controller configuration for a single controller
///
/// This function adds a controller configuration by looking up its mapping
/// in the game controller database and storing it in the SDL_CONTROLLER_CONFIG
/// static variable.
///
/// # Arguments
/// * `index` - The index position for the controller
/// * `guid` - The GUID of the controller to add
///
/// # Returns
/// * `Ok(Some(Controller))` - If the controller was successfully added with its configuration
/// * `Ok(None)` - If the controller was not found in the database or is already configured
/// * `Err(Box<dyn std::error::Error>)` - If there's an error accessing the mutex or if the maximum number of controllers (8) is reached
pub fn add_sdl_controller_config(
    index: usize,
    guid: &str,
) -> Result<Option<Controller>, Box<dyn std::error::Error>> {
    // Check if the controller with this index is already configured
    if is_controller_configured(index) {
        debug!("Controller with index {} is already configured", index);
        return Ok(None); // Return None since it wasn't added (already exists)
    }

    // Get configuration from database
    if let Ok(Some(mapping_data)) = find_gamecontroller_db(guid) {
        // Check if we've reached the maximum number of controllers (8)
        let sdl_controllers_config =
            SDL_CONTROLLER_CONFIG.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        let mut sdl_controllers_config_guard = sdl_controllers_config.lock().map_err(|e| {
            Box::<dyn std::error::Error>::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to lock SDL controller mutex: {}", e),
            ))
        })?;

        // Check if we already have 8 controllers
        if sdl_controllers_config_guard.len() >= 8 {
            return Err("Maximum number of controllers (8) reached. Cannot add more.".into());
        }

        // Parse the mapping data to get name and inputs
        let (name, inputs) = parse_controller_mapping_data(&mapping_data);

        let controller = Controller {
            guid: guid.to_string(),
            name,
            inputs,
        };

        // Add the controller configuration using index as key
        sdl_controllers_config_guard.insert(index, controller.clone());
        Ok(Some(controller))
    } else {
        debug!("Controller mapping not found for GUID: {}", guid);
        Ok(None) // Return None since mapping was not found
    }
}

/// Removes SDL controller configuration(s)
///
/// This function removes controller configuration(s) from the
/// SDL_CONTROLLER_CONFIG static variable.
/// If a specific GUID is provided, removes only that controller.
/// If None is provided, removes all controllers.
///
/// # Arguments
/// * `guid_opt` - Optional GUID of controller to remove. If None, removes all controllers.
///
/// # Returns
/// * `Ok(Vec<Controller>)` - Vector of controllers that were successfully removed
/// * `Err(Box<dyn std::error::Error>)` - If there's an error accessing the mutex
pub fn remove_sdl_controller_config(
    guid_opt: Option<&str>,
) -> Result<Vec<Controller>, Box<dyn std::error::Error>> {
    // Get access to the SDL_CONTROLLER_CONFIG variable
    let sdl_controllers_config =
        SDL_CONTROLLER_CONFIG.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
    let mut sdl_controllers_config_guard = sdl_controllers_config.lock().map_err(|e| {
        Box::<dyn std::error::Error>::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to lock SDL controller mutex: {}", e),
        ))
    })?;

    let successfully_removed = match guid_opt {
        Some(guid) => {
            // Remove specific controller configuration by GUID
            let mut removed_controllers = Vec::new();
            // Find the index that contains the controller with the matching GUID
            let indices_to_remove: Vec<usize> = sdl_controllers_config_guard
                .iter()
                .filter(|(_, controller)| controller.guid == guid)
                .map(|(index, _)| *index)
                .collect();
            
            for index in indices_to_remove {
                if let Some(removed_controller) = sdl_controllers_config_guard.remove(&index) {
                    removed_controllers.push(removed_controller);
                }
            }
            
            if removed_controllers.is_empty() {
                debug!("Controller with GUID {} was not found for removal", guid);
            }
            removed_controllers
        }
        None => {
            // Remove all controller configurations
            let all_controllers: Vec<Controller> =
                sdl_controllers_config_guard.values().cloned().collect();
            sdl_controllers_config_guard.clear();
            all_controllers
        }
    };

    Ok(successfully_removed)
}

/// Gets all controller configurations
///
/// This function returns all controller configurations stored in the
/// SDL_CONTROLLER_CONFIG variable.
///
/// # Returns
/// A HashMap mapping controller indices to Controller objects representing all configured controllers
pub fn get_sdl_controller_config() -> HashMap<usize, Controller> {
    if let Some(sdl_controllers_config) = SDL_CONTROLLER_CONFIG.get() {
        if let Ok(guard) = sdl_controllers_config.lock() {
            // Return all controllers as a HashMap
            guard.clone()
        } else {
            HashMap::new()
        }
    } else {
        HashMap::new()
    }
}

/// This function checks if a controller with the specified index is
/// currently in the SDL_CONTROLLER_CONFIG variable.
/// NOTE: This function is primarily for internal use.
///
/// # Arguments
/// * `index` - The index of the controller to check
///
/// # Returns
/// * `true` - If the controller with the specified index is configured
/// * `false` - If the controller is not configured
pub fn is_controller_configured(index: usize) -> bool {
    if let Some(sdl_controllers_config) = SDL_CONTROLLER_CONFIG.get() {
        if let Ok(guard) = sdl_controllers_config.lock() {
            return guard.contains_key(&index);
        }
    }
    false
}
