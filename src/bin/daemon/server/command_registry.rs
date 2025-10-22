//! Command Registry Module
//!
//! This module provides a flexible command registration and execution system for the regmsg daemon.
//! It allows dynamic registration of commands with different argument patterns and execution behaviors.
//! The system includes specialized command handlers for different use cases like screen management.

use log::{debug, info, warn};
use std::collections::HashMap;
use std::fmt;

/// Result type for command execution
///
/// Represents the result of executing a command, containing either a success message string
/// or a command error.
pub type CommandResult = Result<String, CommandError>;

/// Error type for command handling
///
/// Enumerates possible errors that can occur during command processing:
/// - InvalidArguments: When commands receive incorrect or missing arguments
/// - UnknownCommand: When an unrecognized command is received
/// - ExecutionError: When command execution fails due to an underlying error
/// - EmptyCommand: When an empty command string is received
#[derive(Debug)]
pub enum CommandError {
    InvalidArguments(String),
    UnknownCommand(String),
    ExecutionError(Box<dyn std::error::Error>),
    EmptyCommand,
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            CommandError::UnknownCommand(cmd) => write!(f, "Unknown command: {}", cmd),
            CommandError::ExecutionError(err) => write!(f, "Execution error: {}", err),
            CommandError::EmptyCommand => write!(f, "Empty command"),
        }
    }
}

impl std::error::Error for CommandError {}

/// Trait for command handlers
///
/// Defines the interface that all command handlers must implement to be registered
/// in the command registry. This allows for different types of commands with different
/// execution patterns and argument handling.
pub trait CommandHandler: Send + Sync {
    /// Execute the command with given arguments
    ///
    /// # Arguments
    /// * `args` - A slice of string arguments to pass to the command
    ///
    /// # Returns
    /// * `CommandResult` - The result of command execution
    fn execute(&self, args: &[&str]) -> CommandResult;

    /// Get command description
    ///
    /// # Returns
    /// * `&str` - A human-readable description of what the command does
    fn description(&self) -> &str;

    /// Get expected argument count (None = any number)
    ///
    /// # Returns
    /// * `Option<usize>` - The expected number of arguments, or None if variable
    fn expected_args(&self) -> Option<usize> {
        None
    }
}

/// Command registry for dynamic command management
///
/// Manages a collection of registered commands, allowing for dynamic registration
/// and execution of commands by name. Provides validation of argument counts
/// and proper error handling for unknown commands.
pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn CommandHandler>>,
}

impl CommandRegistry {
    /// Create a new command registry
    ///
    /// # Returns
    /// * `CommandRegistry` - A new empty command registry
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// Register a command handler
    ///
    /// # Arguments
    /// * `name` - The name of the command to register
    /// * `handler` - The command handler to register
    pub fn register<S: Into<String>>(&mut self, name: S, handler: Box<dyn CommandHandler>) {
        let name = name.into();
        info!("Registering command: {}", name);
        self.commands.insert(name, handler);
    }

    /// Handle a command string
    ///
    /// Parses and executes a command from a string, validating argument counts
    /// and returning appropriate results or errors.
    ///
    /// # Arguments
    /// * `cmdline` - The command line string to execute
    ///
    /// # Returns
    /// * `CommandResult` - The result of command execution
    pub fn handle(&self, cmdline: &str) -> CommandResult {
        debug!("Handling command: '{}'", cmdline);

        let parts: Vec<&str> = cmdline.split_whitespace().collect();
        if parts.is_empty() {
            warn!("Received empty command");
            return Err(CommandError::EmptyCommand);
        }

        let (cmd, args) = parts.split_first().unwrap();

        match self.commands.get(*cmd) {
            Some(handler) => {
                // Validate argument count if specified
                if let Some(expected) = handler.expected_args() {
                    if args.len() != expected {
                        return Err(CommandError::InvalidArguments(format!(
                            "{} expects {} arguments, got {}",
                            cmd,
                            expected,
                            args.len()
                        )));
                    }
                }

                info!("Executing command: {} with {} args", cmd, args.len());
                handler.execute(args)
            }
            None => {
                warn!("Unknown command: {}", cmd);
                Err(CommandError::UnknownCommand(cmd.to_string()))
            }
        }
    }

    /// List all registered commands with descriptions
    ///
    /// # Returns
    /// * `String` - A formatted string listing all registered commands and their descriptions
    pub fn list_commands(&self) -> String {
        let mut commands: Vec<_> = self.commands.iter().collect();
        commands.sort_by_key(|(name, _)| *name);

        commands
            .iter()
            .map(|(name, handler)| format!("{}: {}", name, handler.description()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Macro to create simple command handlers
///
/// Creates a command handler that takes no arguments and returns a string result.
///
/// # Arguments
/// * `$name` - Command name (not used in current implementation)
/// * `$desc` - Command description
/// * `$func` - Function to execute
#[macro_export]
macro_rules! simple_command {
    ($name:expr, $desc:expr, $func:expr) => {
        Box::new($crate::server::command_registry::SimpleCommand {
            description: $desc.to_string(),
            executor: Box::new($func),
        })
    };
}

/// Macro to create command handlers with arguments
///
/// Creates a command handler that takes a specific number of arguments.
///
/// # Arguments
/// * `$name` - Command name
/// * `$desc` - Command description
/// * `$args` - Expected number of arguments
/// * `$func` - Function to execute with arguments
#[macro_export]
macro_rules! arg_command {
    ($name:expr, $desc:expr, $args:expr, $func:expr) => {
        Box::new(ArgCommand {
            name: $name.to_string(),
            description: $desc.to_string(),
            expected_args: $args,
            executor: Box::new($func),
        })
    };
}

/// Simple command handler (no arguments)
///
/// Handles commands that take no arguments and return a string result.
/// These commands execute a function that returns a Result<String, Box<dyn Error>>.
pub struct SimpleCommand {
    pub description: String,
    pub executor: Box<dyn Fn() -> Result<String, Box<dyn std::error::Error>> + Send + Sync>,
}

impl CommandHandler for SimpleCommand {
    fn execute(&self, _args: &[&str]) -> CommandResult {
        match (self.executor)() {
            Ok(result) => Ok(result),
            Err(err) => Err(CommandError::ExecutionError(err)),
        }
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn expected_args(&self) -> Option<usize> {
        Some(0)
    }
}

/// Command handler with arguments
///
/// Handles commands that take a specific number of arguments and return a unit result.
/// The executor function returns Result<(), Box<dyn Error>>, and the handler returns
/// a success message on completion.
pub struct ArgCommand {
    pub name: String,
    pub description: String,
    pub expected_args: usize,
    pub executor: Box<dyn Fn(&[&str]) -> Result<(), Box<dyn std::error::Error>> + Send + Sync>,
}

impl CommandHandler for ArgCommand {
    fn execute(&self, args: &[&str]) -> CommandResult {
        match (self.executor)(args) {
            Ok(_) => Ok(format!("{} executed successfully", self.name)),
            Err(err) => Err(CommandError::ExecutionError(err)),
        }
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn expected_args(&self) -> Option<usize> {
        Some(self.expected_args)
    }
}

/// Screen command handler (with optional screen parameter)
///
/// Handles commands that operate on a screen, with an optional screen parameter.
/// The executor function takes an Option<&str> representing the screen identifier.
pub struct ScreenCommand {
    description: String,
    executor: Box<dyn Fn(Option<&str>) -> Result<String, Box<dyn std::error::Error>> + Send + Sync>,
}

impl CommandHandler for ScreenCommand {
    fn execute(&self, args: &[&str]) -> CommandResult {
        let screen = if args.is_empty() { None } else { Some(args[0]) };

        match (self.executor)(screen) {
            Ok(result) => Ok(result),
            Err(err) => Err(CommandError::ExecutionError(err)),
        }
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// Screen setter command handler
///
/// Handles commands that set properties on a screen, taking a value and optional screen parameter.
/// The executor function takes an Option<&str> for the screen and a &str for the value.
pub struct ScreenSetterCommand {
    description: String,
    executor:
        Box<dyn Fn(Option<&str>, &str) -> Result<(), Box<dyn std::error::Error>> + Send + Sync>,
}

impl CommandHandler for ScreenSetterCommand {
    fn execute(&self, args: &[&str]) -> CommandResult {
        if args.is_empty() {
            return Err(CommandError::InvalidArguments(
                "Missing required argument".to_string(),
            ));
        }

        let value = args[0];
        let screen = if args.len() > 1 { Some(args[1]) } else { None };

        match (self.executor)(screen, value) {
            Ok(_) => Ok(format!("Set to {}", value)),
            Err(err) => Err(CommandError::ExecutionError(err)),
        }
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn expected_args(&self) -> Option<usize> {
        None // Variable arguments
    }
}

/// Helper to create screen command handlers
///
/// Creates a ScreenCommand instance with the provided description and executor function.
///
/// # Arguments
/// * `description` - The command description
/// * `executor` - The function to execute when the command is called
///
/// # Returns
/// * `Box<dyn CommandHandler>` - A boxed command handler
pub fn screen_command<F>(description: &str, executor: F) -> Box<dyn CommandHandler>
where
    F: Fn(Option<&str>) -> Result<String, Box<dyn std::error::Error>> + Send + Sync + 'static,
{
    Box::new(ScreenCommand {
        description: description.to_string(),
        executor: Box::new(executor),
    })
}

/// Helper to create screen setter command handlers
///
/// Creates a ScreenSetterCommand instance with the provided description and executor function.
///
/// # Arguments
/// * `description` - The command description
/// * `executor` - The function to execute when the command is called
///
/// # Returns
/// * `Box<dyn CommandHandler>` - A boxed command handler
pub fn screen_setter_command<F>(description: &str, executor: F) -> Box<dyn CommandHandler>
where
    F: Fn(Option<&str>, &str) -> Result<(), Box<dyn std::error::Error>> + Send + Sync + 'static,
{
    Box::new(ScreenSetterCommand {
        description: description.to_string(),
        executor: Box::new(executor),
    })
}
