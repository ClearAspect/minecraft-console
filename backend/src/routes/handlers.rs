//! HTTP request handlers for the application.
//!
//! This file contains the implementation of HTTP handlers for various
//! endpoints like starting/stopping the server and checking status.

use crate::state::AppState;
use crate::websocket::ws_index;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

/// HTTP handler to start the Minecraft server.
///
/// # Returns
/// * Success response if the server was started successfully
/// * Error response with details if the server failed to start
#[derive(Deserialize)]
pub struct StartRequest {
    pub file_path: String,
}

pub async fn start_handler(
    state: web::Data<Arc<Mutex<AppState>>>,
    req: web::Json<StartRequest>,
) -> impl Responder {
    let mut app_state = state.lock().unwrap();
    match app_state.start_minecraft(Some(req.file_path.clone())).await {
        Ok(_) => HttpResponse::Ok().body("Minecraft server started."),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error starting server: {}", e)),
    }
}

/// HTTP handler to stop the Minecraft server.
///
/// # Returns
/// * Success response if the server was stopped successfully
/// * Error response with details if the server failed to stop
pub async fn stop_handler(state: web::Data<Arc<Mutex<AppState>>>) -> impl Responder {
    let mut app_state = state.lock().unwrap();
    match app_state.stop_minecraft().await {
        Ok(_) => HttpResponse::Ok().body("Minecraft server stopped."),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error stopping server: {}", e)),
    }
}

/// HTTP handler to check the server status.
///
/// # Returns
/// * Response indicating whether the server is running or not
pub async fn status_handler(state: web::Data<Arc<Mutex<AppState>>>) -> impl Responder {
    let app_state = state.lock().unwrap();
    if app_state.is_running() {
        HttpResponse::Ok().body("Minecraft server is running.")
    } else {
        HttpResponse::Ok().body("Minecraft server is not running.")
    }
}

/// Configures the application routes.
///
/// # Arguments
/// * `cfg` - Service config to register routes with
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/start").route(web::post().to(start_handler)));
    cfg.service(web::resource("/stop").route(web::post().to(stop_handler)));
    cfg.service(web::resource("/status").route(web::get().to(status_handler)));
    cfg.service(web::resource("/ws").route(web::get().to(ws_index)));
}
