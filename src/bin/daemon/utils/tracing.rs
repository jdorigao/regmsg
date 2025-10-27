//! Tracing Utilities Module
//!
//! This module contains tracing functionality for the regmsg daemon,
//! including logging configuration with file output.

use crate::config::DEFAULT_LOG_PATH;
use std::fs::OpenOptions;
use std::sync::Once;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

static mut WORKER_GUARD: Option<WorkerGuard> = None;
static INIT: Once = Once::new();

/// Initializes the tracing subscriber with file and console output
/// Uses DEFAULT_LOG_PATH from config for the log file location
pub fn setup_tracing() {
    INIT.call_once(|| {
        // Open log file for appending
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(DEFAULT_LOG_PATH)
            .expect("Failed to open log file");

        // Create non-blocking writer for better performance
        let (non_blocking, guard) = tracing_appender::non_blocking::NonBlocking::new(file);
        
        // Store the guard in a static variable to ensure it lives for the duration of the program
        unsafe {
            WORKER_GUARD = Some(guard);
        }

        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info")); // Default to info level if not set

        // Configure file layer
        let file_layer = fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_filter(env_filter.clone());

        // Configure stdout layer for console output
        let stdout_layer = fmt::layer()
            .with_writer(std::io::stdout)
            .with_ansi(true)
            .with_filter(env_filter);

        // Set up the subscriber with both file and console output
        tracing_subscriber::registry()
            .with(file_layer)
            .with(stdout_layer)
            .try_init()
            .expect("Failed to initialize tracing subscriber");
    });
}

