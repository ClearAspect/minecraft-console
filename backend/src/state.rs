// Defines a struct (e.g., AppState) to hold shared data like the process handle, making it accessible across different route handlers.

use crate::server::MinecraftServer;
use std::collections::HashMap;
use std::io::Result;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

// Unique ID counter for WebSocket clients
static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(1);

/// AppState holds the shared state for your application.
pub struct AppState {
    /// An optional instance of the Minecraft server.
    pub minecraft_server: Option<MinecraftServer>,
    /// A sender for forwarding log messages.
    pub log_sender: UnboundedSender<String>,
    /// Map of connected WebSocket clients
    subscribers: HashMap<usize, UnboundedSender<String>>,
}

impl AppState {
    /// Creates a new instance of AppState with the provided log sender.
    pub fn new(log_sender: UnboundedSender<String>) -> Self {
        AppState {
            minecraft_server: None,
            log_sender,
            subscribers: HashMap::new(),
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

    /// Sends a command to the Minecraft server console.
    pub async fn send_command(&mut self, command: &str) -> Result<()> {
        if let Some(server) = &mut self.minecraft_server {
            server.send_command(command).await
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Minecraft server is not running",
            ))
        }
    }

    /// Registers a new WebSocket client and returns a channel for receiving logs
    pub fn register_client(&mut self) -> (usize, UnboundedReceiver<String>) {
        let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::SeqCst);
        let (sender, client_receiver) = unbounded_channel();
        self.subscribers.insert(client_id, sender);
        println!(
            "[WebSocket]: Client #{} connected. Total clients: {}",
            client_id,
            self.subscribers.len()
        );
        return (client_id, client_receiver);
    }

    /// Unregisters a WebSocket client when they disconnect
    pub fn unregister_client(&mut self, client_id: usize) {
        if self.subscribers.remove(&client_id).is_some() {
            println!(
                "[WebSocket]: Client #{} disconnected. Total clients: {}",
                client_id,
                self.subscribers.len()
            );
        }
    }

    /// Broadcast a message to all connected WebSocket clients
    pub fn broadcast_log(&mut self, message: String) {
        // Only log client count if we have subscribers
        if !self.subscribers.is_empty() {
            // Track any clients that need to be disconnected
            let mut disconnected_clients = Vec::new();

            // For all the clients in the subscribers map
            // we send the message
            // If the send fails, we log the error and mark the client for disconnection
            // This is to avoid sending messages to clients that are no longer connected
            for (&client_id, client_receiver) in &self.subscribers {
                match client_receiver.send(message.clone()) {
                    Ok(_) => {} // Success case - no need to log every message
                    Err(e) => {
                        println!(
                            "[WebSocket]: Error sending log to client #{}: {:?}",
                            client_id, e
                        );
                        disconnected_clients.push(client_id);
                    }
                }
            }

            // Clean up disconnected clients
            for client_id in disconnected_clients {
                println!(
                    "[WebSocket]: Client #{} disconnected due to send failure",
                    client_id
                );
                self.unregister_client(client_id);
            }
        }
    }
}
