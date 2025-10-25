// Timeloop - A deterministic time-loop RPG server
// Architecture: REST API -> Game Engine -> Data Models -> Storage

mod models;
mod api;
mod storage;
mod game_engine;

use std::sync::Arc;
use tokio::sync::RwLock;

use api::handlers::AppState;
use api::routes::create_router;
use api::middleware::{cors_layer, tracing_layer};
use storage::{GameDefinitionsLoader, save_manager::SaveManager};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    tracing::info!("Starting Timeloop server...");
    
    // Initialize game definitions loader (empty for now - will load from files later)
    let definitions = Arc::new(GameDefinitionsLoader::new());
    tracing::info!("Game definitions loader initialized");
    
    // Initialize save manager
    let save_manager = Arc::new(SaveManager::new("./saves"));
    tracing::info!("Save manager initialized with directory: ./saves");
    
    // Create shared application state
    let app_state = AppState {
        game_state: Arc::new(RwLock::new(None)),
        definitions: definitions.clone(),
        save_manager: save_manager.clone(),
    };
    
    // Create router with all endpoints
    let app = create_router(app_state)
        .layer(cors_layer())
        .layer(tracing_layer());
    
    // Start server
    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");
    
    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("Health check: http://{}/health", addr);
    tracing::info!("API endpoints: http://{}/api/*", addr);
    
    axum::serve(listener, app)
        .await
        .expect("Server error");
}
