// Timeloop Server - A deterministic time-loop RPG REST API server
// Architecture: REST API -> Business Logic -> Persistence -> Storage

use std::sync::Arc;
use std::path::Path;
use tokio::sync::RwLock;

use timeloop::api::handlers::AppState;
use timeloop::api::routes::create_router;
use timeloop::api::middleware::{cors_layer, tracing_layer};
use timeloop::storage::{GameDefinitionsLoader, save_manager::SaveManager};

// New: Import business logic and persistence layers
use timeloop::business::definitions::{AttributeServiceImpl, CardServiceImpl};
use timeloop::business::validation::{AttributeValidatorImpl, CardValidatorImpl};
use timeloop::persistence::{FileAttributePersistence, AttributePersistence};
use timeloop::persistence::file_storage::FileCardPersistence;

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
    
    // === NEW: Initialize business logic layer with dependency injection ===
    
    // 1. Create persistence layer (file-based)
    let attributes_file = data_path.join("attributes.json");
    let attribute_persistence: Arc<dyn AttributePersistence> = Arc::new(
        FileAttributePersistence::new(attributes_file.clone())
    );
    tracing::info!("Attribute persistence initialized: {:?}", attributes_file);
    
    // 2. Create validator
    let attribute_validator = Arc::new(AttributeValidatorImpl::new());
    tracing::info!("Attribute validator initialized");
    
    // 3. Create business service (injecting persistence and validator)
    let attribute_service = Arc::new(AttributeServiceImpl::new(
        attribute_persistence,
        attribute_validator,
    ));
    tracing::info!("Attribute service initialized with dependency injection");
    
    // 4. Create card persistence layer (file-based)
    let cards_file = data_path.join("cards.json");
    let card_persistence = Arc::new(FileCardPersistence::new(cards_file.clone()));
    tracing::info!("Card persistence initialized: {:?}", cards_file);
    
    // 5. Create card validator
    let card_validator = Arc::new(CardValidatorImpl::new());
    tracing::info!("Card validator initialized");
    
    // 6. Create card service (injecting persistence and validator)
    let card_service = Arc::new(CardServiceImpl::new(
        card_persistence,
        card_validator,
    ));
    tracing::info!("Card service initialized with dependency injection");
    
    // === END NEW ===
    
    // Create shared application state
    let app_state = AppState {
        game_state: Arc::new(RwLock::new(None)),
        definitions: definitions.clone(),
        save_manager: save_manager.clone(),
        attribute_service, // Inject service into app state
        card_service, // Inject card service into app state
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
