// Setup the Actix-web server, configure shared state, and load routes

use actix_web::{web, App, HttpServer};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::unbounded_channel;

mod routes;
mod server;
mod state;
mod websocket;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create a channel for log messages.
    let (log_sender, _log_receiver) = unbounded_channel::<String>();

    // Initialize the shared state.
    let state = Arc::new(Mutex::new(state::AppState::new(log_sender)));

    // Configure and run your Actix-web server.
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .configure(routes::init_routes) // Assuming your routes module configures endpoints.
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
