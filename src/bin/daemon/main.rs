#![cfg(feature = "daemon")]

mod screen;
mod server;

use async_std::channel::{Sender, bounded};
use async_std::stream::StreamExt;
use log::info;
use server::server::DaemonServer;
use signal_hook::consts::signal::*;
use signal_hook_async_std::Signals;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger with flexible configuration via environment variables
    env_logger::init();

    let mut daemon_server = DaemonServer::new()?;

    // Canal para shutdown gracioso
    let (shutdown_tx, shutdown_rx) = bounded(1);

    // Configurar tratamento de sinais
    let signals = Signals::new(&[SIGTERM, SIGINT])?;
    let handle = signals.handle();

    let signal_task = async_std::task::spawn(handle_signals(signals, shutdown_tx));

    // Executar servidor
    let result = daemon_server
        .run(
            screen::list_modes,
            screen::list_outputs,
            screen::current_mode,
            screen::current_output,
            screen::current_resolution,
            screen::current_rotation,
            screen::current_refresh,
            screen::current_backend,
            screen::set_mode,
            screen::set_output,
            screen::set_rotation,
            screen::get_screenshot,
            screen::map_touch_screen,
            screen::min_to_max_resolution,
            shutdown_rx,
        )
        .await;

    // Cleanup
    handle.close();
    signal_task.await;
    daemon_server.shutdown().await?;

    result
}

async fn handle_signals(mut signals: Signals, shutdown_tx: Sender<()>) {
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
