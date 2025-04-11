// Setup the Actix-web server, configure shared state, and load routes

use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};
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

    println!("Starting server on http://127.0.0.1:8080");

    // Configure and run your Actix-web server.
    HttpServer::new(move || {
        // Configure CORS for frontend communication
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(state.clone()))
            .configure(routes::init_routes)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
