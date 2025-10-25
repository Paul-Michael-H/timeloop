// API route definitions and router setup
// This maps HTTP endpoints to handlers

use axum::{
    routing::{get, post, put},
    Router,
};

use super::handlers::*;

/// Create the main API router with all endpoints
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // Game session endpoints
        .route("/api/game/new", post(create_new_game))
        .route("/api/game/current", get(get_current_game))
        .route("/api/game/tick", post(advance_tick))
        .route("/api/game/save", put(save_game))
        .route("/api/game/saves", get(list_saves))
        
        // Action endpoints
        .route("/api/game/training", post(set_training))
        .route("/api/game/affinities", post(acquire_affinity))
        
        // Definition endpoints
        .route("/api/definitions/attributes", get(list_attributes))
        .route("/api/definitions/affinities", get(list_affinities))
        .route("/api/definitions/effects", get(list_effects))
        
        .with_state(state)
}
