use super::command_handler::{CommandResult, CommandError};
use log::{debug, error, warn};

/// Format a command result into a string response
/// 
/// # Arguments
/// * `result` - The command result to format
/// 
/// # Returns
/// * `String` - The formatted response string
pub fn format_response(result: CommandResult) -> String {
    match result {
        Ok(msg) => {
            debug!("Command executed successfully: '{}'", msg);
            msg
        },
        Err(CommandError::ExecutionError(err)) => {
            error!("Command execution error: {}", err);
            format!("Error: {}", err)
        },
        Err(err) => {
            warn!("Command error: {}", err);
            format!("Error: {}", err)
        },
    }
}