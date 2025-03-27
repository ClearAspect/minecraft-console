// Defines a struct (e.g., AppState) to hold shared data like the process handle, making it accessible across different route handlers.

use crate::server::MinecraftServer;
use std::io::Result;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// AppState holds the shared state for your application.
pub struct AppState {
    /// An optional instance of the Minecraft server.
    pub minecraft_server: Option<MinecraftServer>,
    /// A sender for forwarding log messages.
    pub log_sender: UnboundedSender<String>,
}

impl AppState {
    /// Creates a new instance of AppState with the provided log sender.
    pub fn new(log_sender: UnboundedSender<String>) -> Self {
        AppState {
            minecraft_server: None,
            log_sender,
        }
    }

    /// Starts the Minecraft server if it isn’t already running.
    ///
    /// This method calls the `MinecraftServer::start` function from `server.rs`
    /// and stores the resulting server instance in the state.
    pub async fn start_minecraft(&mut self) -> Result<()> {
        if self.minecraft_server.is_none() {
            let server = MinecraftServer::start(self.log_sender.clone()).await?;
            self.minecraft_server = Some(server);
        }
        Ok(())
    }

    /// Stops the Minecraft server if it is currently running.
    pub async fn stop_minecraft(&mut self) -> Result<()> {
        if let Some(server) = &mut self.minecraft_server {
            server.stop().await?;
            self.minecraft_server = None;
        }
        Ok(())
    }

    /// Returns true if the Minecraft server is currently running.
    pub fn is_running(&self) -> bool {
        self.minecraft_server.is_some()
    }

    /// Creates a new log receiver for WebSocket connections
    pub fn create_log_receiver(&self) -> UnboundedReceiver<String> {
        let (sender, receiver) = unbounded_channel();
        // TODO: Store sender in a Vec of subscribers to broadcast to multiple clients
        receiver
    }
}
