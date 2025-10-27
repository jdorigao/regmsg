//! Server Module
//!
//! This module implements the ZeroMQ-based server for the regmsg daemon.
//! It provides a communication interface between clients and the screen management functions
//! through a command registry system. The server handles incoming commands,
//! processes them using registered handlers, and returns appropriate responses.

use super::command_registry::{CommandError, CommandRegistry};
use super::commands;
use crate::config;
use async_std::channel::Receiver;
use futures::FutureExt;
use log::{debug, error, info, warn};
use std::fs;
use std::time::Duration;
use zeromq::prelude::*;
use zeromq::{RepSocket, ZmqMessage};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Maximum message size (1MB)
///
/// Defines the maximum allowed size for incoming messages to prevent memory exhaustion.
const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

/// Maximum retry attempts for sending replies
///
/// Number of times the server will attempt to send a reply before giving up.
const MAX_SEND_RETRIES: usize = 3;

/// Main daemon server structure that handles ZeroMQ communication
///
/// This struct manages the ZeroMQ socket and command registry, providing
/// the interface between clients and the screen management functions.
pub struct DaemonServer {
    /// ZeroMQ reply socket for communication with clients
    socket: RepSocket,
    /// Command registry for dynamic command handling
    registry: CommandRegistry,
}

impl DaemonServer {
    /// Create a new daemon server instance
    ///
    /// This function initializes the server by:
    /// - Removing any existing socket file
    /// - Creating and binding a new ZeroMQ REP socket
    /// - Setting appropriate file permissions on Unix systems
    /// - Initializing the command registry with all available commands
    ///
    /// # Returns
    /// * `Result<DaemonServer, Box<dyn std::error::Error>>` - A new daemon server instance or an error
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Remove existing socket if present
        let _ = fs::remove_file(config::DEFAULT_SOCKET_PATH);

        let mut socket = RepSocket::new();

        // Use blocking operation for bind to ensure it completes
        async_std::task::block_on(async {
            info!("Binding socket to ipc://{}", config::DEFAULT_SOCKET_PATH);
            socket
                .bind(&format!("ipc://{}", config::DEFAULT_SOCKET_PATH))
                .await
        })?;

        // Set appropriate permissions for the socket (Unix only)
        #[cfg(unix)]
        {
            if let Ok(metadata) = fs::metadata(config::DEFAULT_SOCKET_PATH) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o660); // rw-rw---- allows user and group access
                if let Err(e) = fs::set_permissions(config::DEFAULT_SOCKET_PATH, perms) {
                    warn!("Failed to set socket permissions: {}", e);
                } else {
                    info!("Socket permissions set to 0o660");
                }
            }
        }

        // Initialize command registry with all available commands
        let registry = commands::init_commands();
        info!(
            "Initialized {} commands",
            registry.list_commands().lines().count()
        );

        info!(
            "Daemon server initialized on ipc://{}",
            config::DEFAULT_SOCKET_PATH
        );

        Ok(DaemonServer { socket, registry })
    }

    /// Shutdown the daemon server gracefully
    ///
    /// This function performs cleanup operations when shutting down the server,
    /// including removing the socket file to ensure a clean state for the next run.
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok if shutdown succeeds, or an error
    pub async fn shutdown(self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initiating graceful shutdown of daemon server");
        if let Err(e) = fs::remove_file(config::DEFAULT_SOCKET_PATH) {
            warn!(
                "Failed to remove socket file {}: {}",
                config::DEFAULT_SOCKET_PATH,
                e
            );
        } else {
            info!(
                "Socket file {} removed successfully",
                config::DEFAULT_SOCKET_PATH
            );
        }
        Ok(())
    }

    /// Run the daemon server loop, processing incoming commands
    ///
    /// This is the main server loop that continuously listens for incoming messages
    /// and handles shutdown signals. It uses a futures select to handle both
    /// incoming commands and shutdown signals concurrently.
    ///
    /// # Arguments
    /// * `shutdown_rx` - Receiver for shutdown signal
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok if the server runs successfully, or an error
    pub async fn run(
        &mut self,
        shutdown_rx: Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting daemon server loop");

        loop {
            // Use select to handle both messages and shutdown signal
            futures::select! {
                msg = self.socket.recv().fuse() => {
                    match msg {
                        Ok(cmdline) => {
                            debug!("Received message from client");
                            if let Err(e) = self.process_message(cmdline).await {
                                error!("Error processing message: {:?}", e);
                            }
                        }
                        Err(e) => {
                            error!("Error receiving message: {:?}", e);
                        }
                    }
                }
                _ = shutdown_rx.recv().fuse() => {
                    info!("Shutdown signal received, stopping server loop");
                    break;
                }
            }
        }

        info!("Daemon server loop stopped");
        Ok(())
    }

    /// Process a received message
    ///
    /// This function extracts the command string from the received message,
    /// processes it using the command registry, and sends back the response.
    /// It handles both successful results and errors appropriately.
    ///
    /// # Arguments
    /// * `cmdline` - The received ZeroMQ message
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok if processing succeeds, or an error
    async fn process_message(
        &mut self,
        cmdline: ZmqMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Extract and validate command string from the message
        let cmdline_str = match self.extract_command(&cmdline) {
            Ok(s) => {
                info!("Received command: '{}'", s);
                s
            }
            Err(e) => {
                warn!("Invalid command received: {}", e);
                let error_msg = format!("Error: {}", e);
                self.send_reply(error_msg).await?;
                return Ok(());
            }
        };

        // Handle the command using the registry
        let result = self.registry.handle(&cmdline_str);

        // Format and send the response back to the client
        let reply = self.format_response(result);
        debug!("Sending reply: '{}'", reply);
        self.send_reply(reply).await
    }

    /// Extract command string from ZeroMQ message with validation
    ///
    /// This function validates the received message by checking its size
    /// and ensuring it contains valid UTF-8 text. It returns the command
    /// string or an appropriate error message.
    ///
    /// # Arguments
    /// * `cmdline` - The ZeroMQ message to extract from
    ///
    /// # Returns
    /// * `Result<String, String>` - The extracted command string or error message
    fn extract_command(&self, cmdline: &ZmqMessage) -> Result<String, String> {
        let frame = cmdline.get(0).ok_or_else(|| {
            warn!("Received empty message");
            "Received empty message".to_string()
        })?;

        if frame.len() > MAX_MESSAGE_SIZE {
            warn!("Message too large: {} bytes", frame.len());
            return Err(format!(
                "Message too large: {} bytes (max: {})",
                frame.len(),
                MAX_MESSAGE_SIZE
            ));
        }

        match String::from_utf8(frame.to_vec()) {
            Ok(s) => {
                debug!("Successfully extracted command string: '{}'", s);
                Ok(s)
            }
            Err(e) => {
                warn!("Invalid UTF-8 message: {}", e);
                Err(format!("Invalid UTF-8 message: {}", e))
            }
        }
    }

    /// Format a command result into a string response
    ///
    /// This function takes the result of command execution and formats it
    /// into a string response that can be sent back to the client.
    /// It handles both success and error cases appropriately.
    ///
    /// # Arguments
    /// * `result` - The command result to format
    ///
    /// # Returns
    /// * `String` - The formatted response string
    fn format_response(&self, result: Result<String, CommandError>) -> String {
        match result {
            Ok(msg) => {
                debug!("Command executed successfully: '{}'", msg);
                msg
            }
            Err(CommandError::ExecutionError(err)) => {
                error!("Command execution error: {}", err);
                format!("Error: {}", err)
            }
            Err(err) => {
                warn!("Command error: {}", err);
                format!("Error: {}", err)
            }
        }
    }

    /// Send a reply to the client with retry logic
    ///
    /// This function attempts to send a reply to the client, with retry logic
    /// in case of temporary failures. It waits between attempts with an
    /// exponentially increasing delay to avoid overwhelming the system.
    ///
    /// # Arguments
    /// * `reply` - The reply string to send
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok if send succeeds, or an error
    async fn send_reply(&mut self, reply: String) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Attempting to send reply: '{}'", reply);

        for attempt in 0..MAX_SEND_RETRIES {
            match self.socket.send(ZmqMessage::from(reply.clone())).await {
                Ok(_) => {
                    if attempt > 0 {
                        info!("Reply sent successfully on attempt {}", attempt + 1);
                    } else {
                        debug!("Reply sent successfully on first attempt");
                    }
                    return Ok(());
                }
                Err(e) if attempt < MAX_SEND_RETRIES - 1 => {
                    warn!("Failed to send reply (attempt {}): {:?}", attempt + 1, e);
                    // Wait with exponential backoff (100ms, 200ms, 300ms, etc.)
                    async_std::task::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
                }
                Err(e) => {
                    error!(
                        "Failed to send reply after {} attempts: {:?}",
                        MAX_SEND_RETRIES, e
                    );
                    return Err(Box::new(e));
                }
            }
        }
        unreachable!()
    }
}


