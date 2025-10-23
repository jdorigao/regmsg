use log::debug;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader};

pub type Guid = String;

#[derive(Debug, Clone)]
pub struct ControllerMapping {
    pub mapping_data: String,
}

pub fn load_gamecontroller_db() -> io::Result<HashMap<Guid, ControllerMapping>> {
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
