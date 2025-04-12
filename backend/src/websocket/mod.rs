//! WebSocket module for real-time communication with clients.
//!
//! This module handles WebSocket connections for the application, enabling
//! real-time console access and bidirectional communication.

mod console_socket;

pub use console_socket::ws_index;
