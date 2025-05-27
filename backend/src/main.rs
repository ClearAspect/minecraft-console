//! Minecraft Console Backend main module.
//!
//! This module initializes and runs the Actix-web server that provides:
//! - HTTP endpoints for controlling the Minecraft server
//! - WebSocket connections for real-time console access
//! - Log forwarding from the Minecraft server to clients

use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::unbounded_channel;

mod routes;
mod server;
mod state;
mod websocket;

/// Main entry point for the application.
///
/// This function:
/// 1. Sets up communication channels for log messages
/// 2. Initializes shared application state
/// 3. Creates a log broadcaster task
/// 4. Configures and starts the Actix-web server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create a channel for log messages.
    let (log_sender, mut log_receiver) = unbounded_channel::<String>();

    // Initialize the shared state.
    let state = Arc::new(Mutex::new(state::AppState::new(log_sender)));

    // Create a log broadcaster task to forward logs to connected clients
    let state_clone = state.clone();
    tokio::spawn(async move {
        println!("Log broadcaster started");

        // Process incoming log messages
        while let Some(log) = log_receiver.recv().await {
            // Forward logs to all connected WebSocket clients
            match state_clone.lock() {
                Ok(mut app_state) => {
                    // Skip empty logs and just newlines to reduce noise
                    let trimmed = log.trim();
                    if !trimmed.is_empty() {
                        // Broadcast the log to the subscribers
                        app_state.broadcast_log(log);
                    } else {
                        // Skip empty messages silently
                    }
                }
                Err(e) => {
                    println!("Error: Could not lock app_state for broadcasting: {:?}", e);
                }
            }
        }

        println!("Log broadcaster terminated - channel closed");
    });

    // Print server startup message
    println!("Starting server on http://0.0.0.0:8080");

    // Configure and run the Actix-web server
    HttpServer::new(move || {
        // Configure CORS for frontend communication
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://192.168.10.208:3000")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .supports_credentials()
            .max_age(3600);

        // Create and configure the application
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(state.clone()))
            .configure(routes::init_routes)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
