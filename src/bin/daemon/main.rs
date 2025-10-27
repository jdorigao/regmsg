#![cfg(feature = "daemon")]

/// Configuration module for centralized settings
mod config;
/// Error handling module with unified error types
mod error;
/// Screen management module providing display configuration functions
mod screen;
/// Server module containing ZeroMQ communication and command handling
mod server;

use async_std::channel::bounded;
use async_std::stream::StreamExt;
use log::info;
use server::server::DaemonServer;
use signal_hook::consts::signal::*;
use signal_hook_async_std::Signals;

/// Main entry point for the regmsg daemon
/// 
/// This function initializes the daemon server, sets up signal handling for graceful shutdown,
/// and runs the main server loop. It handles SIGTERM and SIGINT signals to allow for proper
/// cleanup when the daemon is stopped.
/// 
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Ok if the daemon runs and shuts down successfully, or an error
#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger with flexible configuration via environment variables
    env_logger::init();

    // Create the daemon server with integrated command registry
    let mut daemon_server = DaemonServer::new()?;

    // Channel for graceful shutdown
    let (shutdown_tx, shutdown_rx) = bounded(1);

    // Configure signal handling for graceful shutdown
    let signals = Signals::new(&[SIGTERM, SIGINT])?;
    let handle = signals.handle();

    // Spawn async task to handle OS signals
    let signal_task = async_std::task::spawn(handle_signals(signals, shutdown_tx));

    // Run the daemon server with shutdown receiver
    let result = daemon_server.run(shutdown_rx).await;

    // Cleanup resources
    handle.close();
    signal_task.await;
    daemon_server.shutdown().await?;

    result
}

/// Asynchronous function to handle system signals for graceful shutdown
/// 
/// This function listens for SIGTERM and SIGINT signals and sends a shutdown signal
/// through the provided channel when either is received, allowing the main server
/// loop to terminate gracefully.
/// 
/// # Arguments
/// * `signals` - The Signals struct that listens for system signals
/// * `shutdown_tx` - The sender channel to signal shutdown
async fn handle_signals(mut signals: Signals, shutdown_tx: async_std::channel::Sender<()>) {
    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM | SIGINT => {
                info!("Received signal {}, initiating shutdown", signal);
                let _ = shutdown_tx.send(()).await;
                break;
            }
            _ => {}
        }
    }
}
