//! Commands Module
//!
//! This module registers all available commands with the command registry.
//! It maps command names to their respective functions in the screen module,
//! providing a clean interface between the ZeroMQ server and screen management functions.

use super::command_registry::{CommandRegistry, screen_command, screen_setter_command};
use crate::controller;
use crate::screen;
use crate::simple_command;

/// Initialize all available commands in the registry
///
/// This function creates and registers all supported commands with the command registry,
/// mapping command names to their respective screen management functions.
/// The commands are categorized into three groups:
/// - Query commands (no arguments): Commands that return information
/// - Screen-aware query commands: Commands that take an optional screen parameter
/// - Setter commands (with arguments): Commands that modify settings
///
/// # Returns
/// * `CommandRegistry` - A registry containing all available commands
pub fn init_commands() -> CommandRegistry {
    let mut registry = CommandRegistry::new();

    registry.register(
        "listCommands",
        simple_command!("listCommands", "List all available commands", || {
            let temp_registry = init_commands();
            Ok(temp_registry.list_commands())
        }),
    );

    // ----------------------------------------------
    // Screen management commands
    // ----------------------------------------------

    registry.register(
        "listOutputs",
        simple_command!("listOutputs", "List all available display outputs", || {
            Ok(screen::list_outputs()?)
        }),
    );

    registry.register(
        "currentOutput",
        simple_command!(
            "currentOutput",
            "Displays the current output (e.g., HDMI, VGA)",
            || Ok(screen::current_output()?)
        ),
    );

    registry.register(
        "currentBackend",
        simple_command!(
            "currentBackend",
            "Displays the current window system",
            || Ok(screen::current_backend()?)
        ),
    );

    registry.register(
        "getScreenshot",
        simple_command!(
            "getScreenshot",
            "Takes a screenshot of the current screen",
            || {
                screen::get_screenshot()?;
                Ok("Screenshot taken".to_string())
            }
        ),
    );

    registry.register(
        "mapTouchScreen",
        simple_command!(
            "mapTouchScreen",
            "Maps the touchscreen to the correct display",
            || {
                screen::map_touch_screen()?;
                Ok("Touchscreen mapped".to_string())
            }
        ),
    );

    registry.register(
        "listModes",
        screen_command("Lists all available outputs (e.g., HDMI, VGA)", |screen| {
            Ok(screen::list_modes(screen)?)
        }),
    );

    registry.register(
        "currentMode",
        screen_command(
            "Displays the current display mode for the specified screen",
            |screen| Ok(screen::current_mode(screen)?),
        ),
    );

    registry.register(
        "currentResolution",
        screen_command(
            "Displays the current resolution for the specified screen",
            |screen| Ok(screen::current_resolution(screen)?),
        ),
    );

    registry.register(
        "currentRotation",
        screen_command(
            "Displays the current screen rotation for the specified screen",
            |screen| Ok(screen::current_rotation(screen)?),
        ),
    );

    registry.register(
        "currentRefresh",
        screen_command(
            "Displays the current refresh rate for the specified screen",
            |screen| Ok(screen::current_refresh(screen)?),
        ),
    );

    registry.register(
        "minToMaxResolution",
        screen_command(
            "Sets the screen resolution to the maximum supported resolution",
            |screen| {
                screen::min_to_max_resolution(screen)?;
                Ok("Resolution set to maximum".to_string())
            },
        ),
    );

    registry.register(
        "setMode",
        screen_setter_command(
            "Sets the display mode for the specified screen (e.g., 1920x1080@60)",
            |screen, mode| Ok(screen::set_mode(screen, mode)?),
        ),
    );

    registry.register(
        "setOutput",
        Box::new(super::command_registry::ArgCommand {
            name: "setOutput".to_string(),
            description: "Sets the output resolution and refresh rate (e.g., WxH@R or WxH)"
                .to_string(),
            expected_args: 1,
            executor: Box::new(|args| Ok(screen::set_output(args[0])?)),
        }),
    );

    registry.register(
        "setRotation",
        screen_setter_command(
            "Sets the screen rotation for the specified screen (0, 90, 180, 270)",
            |screen, rotation| {
                // Validate rotation value to ensure it's one of the allowed values
                if !["0", "90", "180", "270"].contains(&rotation) {
                    return Err(format!(
                        "Invalid rotation: '{}'. Valid options are: 0, 90, 180, 270",
                        rotation
                    )
                    .into());
                }
                Ok(screen::set_rotation(screen, rotation)?)
            },
        ),
    );

    // ----------------------------------------------
    // Controller management commands
    // ----------------------------------------------

    registry.register(
        "addController",
        Box::new(super::command_registry::ArgCommand {
            name: "addController".to_string(),
            description: "Adds controller to the system by index and GUID".to_string(),
            expected_args: 2,
            executor: Box::new(|args| {
                let index_str = args[0].trim();
                let guid = args[1].trim();

                // Parse the index
                let index = index_str.parse::<usize>().map_err(|_| {
                    format!("Invalid index: {}. Index must be a positive integer.", index_str)
                })?;

                if guid.is_empty() {
                    return Err("GUID cannot be empty.".into());
                }

                let _ = controller::add_controller(index, guid)?;
                Ok(())
            }),
        }),
    );

    registry.register(
        "removeController",
        Box::new(super::command_registry::ArgCommand {
            name: "removeController".to_string(),
            description: "Remove controller. Specify 1 GUID to remove a specific controller"
                .to_string(),
            expected_args: 1, // Must have exactly 1 arg
            executor: Box::new(|args| {
                // Must have exactly 1 argument
                if args.len() != 1 {
                    return Err("Expected exactly 1 controller GUID.".into());
                }

                let guid = args[0].trim();

                if guid.is_empty() {
                    return Err("GUID cannot be empty.".into());
                }

                controller::remove_controller(guid)?;

                // Return success regardless of whether controllers were found
                Ok(())
            }),
        }),
    );

    registry.register(
        "getController",
        simple_command!("getController", "Get all controller configurations", || {
            let controllers_json = controller::get_controller()?;

            // Check if the JSON indicates no controllers
            let result = if controllers_json == "{}" {
                "No controllers configured".to_string()
            } else {
                controllers_json
            };
            Ok(result)
        }),
    );

    registry
}
