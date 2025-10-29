//! Server Module
//!
//! This module contains the core server components for the regmsg daemon.
//! It provides a modular architecture for handling client requests and communicating
//! with display backends through a ZeroMQ interface.
//!
//! The server module is organized into three main components:
//! - command_registry: Manages dynamic command registration and execution
//! - commands: Initializes and registers all available commands
//! - server: Implements the ZeroMQ communication layer

/// Command registry module - manages dynamic command registration and execution
pub mod command_registry;

/// Commands module - initializes and registers all available commands with the registry
pub mod commands;

/// Server module - implements the ZeroMQ communication layer and message handling
pub mod server;

/// Server tests module - contains comprehensive tests for the server components
#[cfg(test)]
mod server_tests;
