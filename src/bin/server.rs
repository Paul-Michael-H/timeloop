// Timeloop Server - A deterministic time-loop RPG REST API server
// Architecture: REST API -> Game Engine -> Data Models -> Storage

use std::sync::Arc;
use std::path::Path;
use tokio::sync::RwLock;

use timeloop::api::handlers::AppState;
use timeloop::api::routes::create_router;
use timeloop::api::middleware::{cors_layer, tracing_layer};
use timeloop::storage::{GameDefinitionsLoader, save_manager::SaveManager};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    tracing::info!("Starting Timeloop server...");
    
    // Load game definitions from JSON files
    let data_path = Path::new("game_data/core");
    let definitions = match GameDefinitionsLoader::load_all_definitions(data_path).await {
        Ok(defs) => {
            let attr_count = defs.list_all_attributes().len();
            tracing::info!("Game definitions loaded successfully");
            tracing::info!("  - {} attributes", attr_count);
            Arc::new(defs)
        }
        Err(e) => {
            tracing::error!("Failed to load game definitions: {:?}", e);
            tracing::warn!("Starting with empty definitions");
            Arc::new(GameDefinitionsLoader::new())
        }
    };
    
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
