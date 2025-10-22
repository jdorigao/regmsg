use super::command_handler;
use super::response_handler;
use async_std::channel::Receiver;
use futures::FutureExt;
use std::fs;
use std::time::Duration;
use zeromq::prelude::*;
use zeromq::{RepSocket, ZmqMessage};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Path for the daemon socket
const SOCKET_PATH: &str = "/var/run/regmsgd.sock";
/// Maximum message size (1MB)
const MAX_MESSAGE_SIZE: usize = 1024 * 1024;
/// Maximum retry attempts for sending replies
const MAX_SEND_RETRIES: usize = 3;

/// Main daemon server structure that handles ZeroMQ communication
pub struct DaemonServer {
    /// ZeroMQ reply socket for communication with clients
    socket: RepSocket,
}

impl DaemonServer {
    /// Create a new daemon server instance
    ///
    /// # Returns
    /// * `Result<DaemonServer, Box<dyn std::error::Error>>` - A new daemon server instance or an error
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Remove existing socket if present
        let _ = fs::remove_file(SOCKET_PATH);

        let mut socket = RepSocket::new();

        // Use blocking operation for bind to ensure it completes
        async_std::task::block_on(async { socket.bind(&format!("ipc://{}", SOCKET_PATH)).await })?;

        // Set appropriate permissions for the socket (Unix only)
        #[cfg(unix)]
        {
            if let Ok(metadata) = fs::metadata(SOCKET_PATH) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o660); // rw-rw----
                let _ = fs::set_permissions(SOCKET_PATH, perms);
            }
        }

        log::info!("Daemon running on ipc://{}", SOCKET_PATH);

        Ok(DaemonServer { socket })
    }

    /// Shutdown the daemon server gracefully
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok if shutdown succeeds, or an error
    pub async fn shutdown(self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Shutting down daemon server");
        let _ = fs::remove_file(SOCKET_PATH);
        Ok(())
    }

    /// Run the daemon server loop, processing incoming commands
    ///
    /// # Arguments
    /// * `list_modes` - Function to list display modes
    /// * `list_outputs` - Function to list display outputs
    /// * `current_mode` - Function to get current display mode
    /// * `current_output` - Function to get current display output
    /// * `current_resolution` - Function to get current resolution
    /// * `current_rotation` - Function to get current rotation
    /// * `current_refresh` - Function to get current refresh rate
    /// * `current_backend` - Function to get current backend
    /// * `set_mode` - Function to set display mode
    /// * `set_output` - Function to set display output
    /// * `set_rotation` - Function to set rotation
    /// * `get_screenshot` - Function to take a screenshot
    /// * `map_touch_screen` - Function to map touchscreen
    /// * `min_to_max_resolution` - Function to set resolution to maximum
    /// * `shutdown_rx` - Receiver for shutdown signal
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok if the server runs successfully, or an error
    pub async fn run(
        &mut self,
        list_modes: command_handler::ListModesFn,
        list_outputs: command_handler::ListOutputsFn,
        current_mode: command_handler::CurrentModeFn,
        current_output: command_handler::CurrentOutputFn,
        current_resolution: command_handler::CurrentResolutionFn,
        current_rotation: command_handler::CurrentRotationFn,
        current_refresh: command_handler::CurrentRefreshFn,
        current_backend: command_handler::CurrentBackendFn,
        set_mode: command_handler::SetModeFn,
        set_output: command_handler::SetOutputFn,
        set_rotation: command_handler::SetRotationFn,
        get_screenshot: command_handler::GetScreenshotFn,
        map_touch_screen: command_handler::MapTouchScreenFn,
        min_to_max_resolution: command_handler::MinToMaxResolutionFn,
        shutdown_rx: Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // Use select to handle both messages and shutdown signal
            futures::select! {
                msg = self.socket.recv().fuse() => {
                    match msg {
                        Ok(cmdline) => {
                            if let Err(e) = self.process_message(
                                cmdline,
                                list_modes,
                                list_outputs,
                                current_mode,
                                current_output,
                                current_resolution,
                                current_rotation,
                                current_refresh,
                                current_backend,
                                set_mode,
                                set_output,
                                set_rotation,
                                get_screenshot,
                                map_touch_screen,
                                min_to_max_resolution,
                            ).await {
                                log::error!("Error processing message: {:?}", e);
                            }
                        }
                        Err(e) => {
                            log::error!("Error receiving message: {:?}", e);
                        }
                    }
                }
                _ = shutdown_rx.recv().fuse() => {
                    log::info!("Shutdown signal received, stopping server loop");
                    break;
                }
            }
        }
        Ok(())
    }

    /// Process a received message
    ///
    /// # Arguments
    /// * `cmdline` - The received ZeroMQ message
    /// * Function pointers for all screen operations
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok if processing succeeds, or an error
    async fn process_message(
        &mut self,
        cmdline: ZmqMessage,
        list_modes: command_handler::ListModesFn,
        list_outputs: command_handler::ListOutputsFn,
        current_mode: command_handler::CurrentModeFn,
        current_output: command_handler::CurrentOutputFn,
        current_resolution: command_handler::CurrentResolutionFn,
        current_rotation: command_handler::CurrentRotationFn,
        current_refresh: command_handler::CurrentRefreshFn,
        current_backend: command_handler::CurrentBackendFn,
        set_mode: command_handler::SetModeFn,
        set_output: command_handler::SetOutputFn,
        set_rotation: command_handler::SetRotationFn,
        get_screenshot: command_handler::GetScreenshotFn,
        map_touch_screen: command_handler::MapTouchScreenFn,
        min_to_max_resolution: command_handler::MinToMaxResolutionFn,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Extract and validate command string
        let cmdline_str = match self.extract_command(&cmdline) {
            Ok(s) => s,
            Err(e) => {
                let error_msg = format!("Error: {}", e);
                self.send_reply(error_msg).await?;
                return Ok(());
            }
        };

        // Handle the command and get result
        let result = command_handler::handle_command(
            &cmdline_str,
            list_modes,
            list_outputs,
            current_mode,
            current_output,
            current_resolution,
            current_rotation,
            current_refresh,
            current_backend,
            set_mode,
            set_output,
            set_rotation,
            get_screenshot,
            map_touch_screen,
            min_to_max_resolution,
        );

        // Format and send the response
        let reply = response_handler::format_response(result);
        self.send_reply(reply).await
    }

    /// Extract command string from ZeroMQ message with validation
    ///
    /// # Arguments
    /// * `cmdline` - The ZeroMQ message to extract from
    ///
    /// # Returns
    /// * `Result<String, String>` - The extracted command string or error message
    fn extract_command(&self, cmdline: &ZmqMessage) -> Result<String, String> {
        let frame = cmdline
            .get(0)
            .ok_or_else(|| "Received empty message".to_string())?;

        if frame.len() > MAX_MESSAGE_SIZE {
            log::warn!("Message too large: {} bytes", frame.len());
            return Err(format!(
                "Message too large: {} bytes (max: {})",
                frame.len(),
                MAX_MESSAGE_SIZE
            ));
        }

        String::from_utf8(frame.to_vec()).map_err(|e| format!("Invalid UTF-8 message: {}", e))
    }

    /// Send a reply to the client with retry logic
    ///
    /// # Arguments
    /// * `reply` - The reply string to send
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok if send succeeds, or an error
    async fn send_reply(&mut self, reply: String) -> Result<(), Box<dyn std::error::Error>> {
        for attempt in 0..MAX_SEND_RETRIES {
            match self.socket.send(ZmqMessage::from(reply.clone())).await {
                Ok(_) => {
                    if attempt > 0 {
                        log::info!("Reply sent successfully on attempt {}", attempt + 1);
                    }
                    return Ok(());
                }
                Err(e) if attempt < MAX_SEND_RETRIES - 1 => {
                    log::warn!("Failed to send reply (attempt {}): {:?}", attempt + 1, e);
                    async_std::task::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
                }
                Err(e) => {
                    log::error!(
                        "Failed to send reply after {} attempts: {:?}",
                        MAX_SEND_RETRIES,
                        e
                    );
                    return Err(Box::new(e));
                }
            }
        }
        unreachable!()
    }
}
