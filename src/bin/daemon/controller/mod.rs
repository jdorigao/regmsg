pub mod controllerdb;
pub use controllerdb::{Guid, add_controllers_to_sdl_db, get_sdl_controller_db};

pub fn add_guids_to_sdl_db(guids: Vec<Guid>) -> Result<(), Box<dyn std::error::Error>> {
    add_controllers_to_sdl_db(guids)
}

pub fn get_sdl_db() -> Result<
    std::sync::Arc<
        std::sync::Mutex<std::collections::HashMap<Guid, controllerdb::ControllerMapping>>,
    >,
    Box<dyn std::error::Error>,
> {
    get_sdl_controller_db()
}
