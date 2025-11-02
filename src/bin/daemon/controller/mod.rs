pub mod controllerdb;

/// Adds a single controller configuration by index and GUID
///
/// This function adds a controller configuration by looking up its mapping
/// in the game controller database and storing it.
///
/// # Arguments
/// * `index` - The index position for the controller (for ordering purposes)
/// * `guid` - The GUID of the controller to add
///
/// # Returns
/// * `Ok(Some(Controller))` - If the controller was successfully added with its configuration
/// * `Ok(None)` - If the controller was not found in the database or is already configured
/// * `Err(Box<dyn std::error::Error>)` - If there's an error accessing the mutex or if the maximum number of controllers (8) is reached
pub fn add_controller(
    index: usize,
    guid: &str,
) -> Result<Option<controllerdb::Controller>, Box<dyn std::error::Error>> {
    controllerdb::add_sdl_controller_config(index, guid)
}

/// Removes a single controller configuration
///
/// This function removes a specific controller configuration from storage.
/// It internally calls the remove function with the provided GUID.
///
/// # Arguments
/// * `guid` - The GUID of the controller to remove
///
/// # Returns
/// * `Ok(())` - If the controller was successfully removed or was not found
/// * `Err(Box<dyn std::error::Error>)` - If there's an error accessing the mutex
pub fn remove_controller(guid: &str) -> Result<(), Box<dyn std::error::Error>> {
    let results = controllerdb::remove_sdl_controller_config(Some(guid))?;

    // Return Ok regardless of whether controllers were found
    let _ = results;
    Ok(())
}

/// Gets all controller configurations as JSON
///
/// This function returns all controller configurations serialized as JSON.
/// The result is a JSON object where keys are controller indices and values contain
/// the controller name and input mappings.
///
/// # Returns
/// A JSON string representing all configured controllers
pub fn get_controller() -> Result<String, Box<dyn std::error::Error>> {
    let controllers = controllerdb::get_sdl_controller_config();
    let json_string = serde_json::to_string(&controllers)?;
    Ok(json_string)
}

#[cfg(test)]
mod controller_tests;
