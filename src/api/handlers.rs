// HTTP request handlers
// This includes: game state queries, action submissions, save/load operations

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::game_engine::simulation::{GameState, GameAction};
use crate::models::common::*;
use crate::storage::{GameDefinitionsLoader, save_manager::SaveManager};

// New: Import business logic trait for method calls
use crate::business::AttributeService;

// ============================================================================
// SHARED STATE
// ============================================================================

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub game_state: Arc<RwLock<Option<GameState>>>,
    pub definitions: Arc<GameDefinitionsLoader>,
    pub save_manager: Arc<SaveManager>,
    // New: Injected business logic services
    // Using concrete type for now - can be made generic later if needed
    pub attribute_service: Arc<crate::business::definitions::AttributeServiceImpl>,
}

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Deserialize)]
pub struct NewGameRequest {
    pub name: String,
    pub starting_attributes: std::collections::HashMap<String, u32>,
}

#[derive(Serialize)]
pub struct GameStateResponse {
    pub character_id: String,
    pub name: String,
    pub current_loop: u32,
    pub current_tick: u64,
    pub attributes: Vec<AttributeInfo>,
    pub affinities: Vec<AffinityInfo>,
}

#[derive(Serialize)]
pub struct AttributeInfo {
    pub id: String,
    pub definition_id: String,
    pub name: String,
    pub value: u32,
    pub training: bool,
}

#[derive(Serialize)]
pub struct AffinityInfo {
    pub id: String,
    pub definition_id: String,
    pub base_mastery_level: u32,
}

#[derive(Deserialize)]
pub struct TrainingRequest {
    pub attribute_id: String,
    pub active: bool,
}

#[derive(Deserialize)]
pub struct AcquireAffinityRequest {
    pub affinity_id: String,
}

#[derive(Serialize)]
pub struct TickResponse {
    pub tick: u64,
    pub trained_attributes: Vec<String>,
    pub mastered_affinities: Vec<String>,
}

// ============================================================================
// GAME SESSION HANDLERS
// ============================================================================

/// POST /api/game/new - Create new game with new character
pub async fn create_new_game(
    State(state): State<AppState>,
    Json(req): Json<NewGameRequest>,
) -> Result<Json<GameStateResponse>, ApiError> {
    // Parse starting attributes
    let mut attributes = std::collections::HashMap::new();
    for (attr_id_str, value) in req.starting_attributes {
        let uuid = uuid::Uuid::parse_str(&attr_id_str)
            .map_err(|_| ApiError::InvalidInput("Invalid attribute ID format".to_string()))?;
        attributes.insert(AttributeId::from(uuid), value);
    }
    
    // Create new character
    let character = crate::models::instances::Character::new(req.name, attributes);
    
    // Create game state
    let game_state = GameState::new(character, (*state.definitions).clone());
    
    // Store in shared state
    let mut state_guard = state.game_state.write().await;
    *state_guard = Some(game_state);
    drop(state_guard);
    
    // Return current state
    get_current_game(State(state)).await
}

/// GET /api/game/current - Get current active character
pub async fn get_current_game(
    State(state): State<AppState>,
) -> Result<Json<GameStateResponse>, ApiError> {
    let state_guard = state.game_state.read().await;
    let game_state = state_guard.as_ref()
        .ok_or(ApiError::NoActiveGame)?;
    
    let character = &game_state.character;
    
    // Build attribute list with names from definitions
    let attributes: Vec<AttributeInfo> = character.attributes.iter().map(|(def_id, attr)| {
        let name = state.definitions.get_attribute_definition(def_id)
            .map(|def| def.name.clone())
            .unwrap_or_else(|| format!("Unknown ({:?})", def_id));
        
        AttributeInfo {
            id: format!("{:?}", attr.id),
            definition_id: format!("{:?}", def_id),
            name,
            value: attr.base_value,
            training: matches!(attr.training_mode, TrainingMode::Active),
        }
    }).collect();
    
    // Build affinity list
    let affinities: Vec<AffinityInfo> = character.affinities.iter().map(|(id, aff)| {
        AffinityInfo {
            id: format!("{:?}", aff.id),
            definition_id: format!("{:?}", id),
            base_mastery_level: aff.base_mastery_level,
        }
    }).collect();
    
    Ok(Json(GameStateResponse {
        character_id: format!("{:?}", character.id),
        name: character.name.clone(),
        current_loop: character.current_loop,
        current_tick: character.current_tick.get(),
        attributes,
        affinities,
    }))
}

/// POST /api/game/tick - Advance game by one tick
pub async fn advance_tick(
    State(state): State<AppState>,
) -> Result<Json<TickResponse>, ApiError> {
    let mut state_guard = state.game_state.write().await;
    let game_state = state_guard.as_mut()
        .ok_or(ApiError::NoActiveGame)?;
    
    let result = game_state.advance_tick();
    
    Ok(Json(TickResponse {
        tick: result.tick.get(),
        trained_attributes: result.trained_attributes.iter()
            .map(|id| format!("{:?}", id))
            .collect(),
        mastered_affinities: result.mastered_affinities.iter()
            .map(|id| format!("{:?}", id))
            .collect(),
    }))
}

/// PUT /api/game/save - Save current game state
pub async fn save_game(
    State(state): State<AppState>,
) -> Result<StatusCode, ApiError> {
    let state_guard = state.game_state.read().await;
    let game_state = state_guard.as_ref()
        .ok_or(ApiError::NoActiveGame)?;
    
    state.save_manager.save_character(&game_state.character).await
        .map_err(|e| ApiError::StorageError(e.to_string()))?;
    
    Ok(StatusCode::OK)
}

/// GET /api/game/saves - List all available saves
pub async fn list_saves(
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, ApiError> {
    let saves = state.save_manager.list_saved_characters().await
        .map_err(|e| ApiError::StorageError(e.to_string()))?;
    
    Ok(Json(saves))
}

// ============================================================================
// ACTION HANDLERS
// ============================================================================

/// POST /api/game/training - Start/stop attribute training
pub async fn set_training(
    State(state): State<AppState>,
    Json(req): Json<TrainingRequest>,
) -> Result<StatusCode, ApiError> {
    let uuid = uuid::Uuid::parse_str(&req.attribute_id)
        .map_err(|_| ApiError::InvalidInput("Invalid attribute ID".to_string()))?;
    let attr_id = AttributeId::from(uuid);
    
    let action = if req.active {
        GameAction::StartTraining { attribute_id: attr_id }
    } else {
        GameAction::StopTraining { attribute_id: attr_id }
    };
    
    let mut state_guard = state.game_state.write().await;
    let game_state = state_guard.as_mut()
        .ok_or(ApiError::NoActiveGame)?;
    
    game_state.execute_action(action)
        .map_err(|e| ApiError::GameError(e.to_string()))?;
    
    Ok(StatusCode::OK)
}

/// POST /api/game/affinities - Acquire new affinity
pub async fn acquire_affinity(
    State(state): State<AppState>,
    Json(req): Json<AcquireAffinityRequest>,
) -> Result<StatusCode, ApiError> {
    let uuid = uuid::Uuid::parse_str(&req.affinity_id)
        .map_err(|_| ApiError::InvalidInput("Invalid affinity ID".to_string()))?;
    let aff_id = AffinityId::from(uuid);
    
    let action = GameAction::AcquireAffinity { affinity_id: aff_id };
    
    let mut state_guard = state.game_state.write().await;
    let game_state = state_guard.as_mut()
        .ok_or(ApiError::NoActiveGame)?;
    
    game_state.execute_action(action)
        .map_err(|e| ApiError::GameError(e.to_string()))?;
    
    Ok(StatusCode::OK)
}

// ============================================================================
// DEFINITION HANDLERS
// ============================================================================

/// GET /api/definitions/attributes - List all attribute definitions
pub async fn list_attributes(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::definitions::AttributeDefinition>>, ApiError> {
    let attrs = state.definitions.list_all_attributes();
    Ok(Json(attrs.into_iter().cloned().collect()))
}

/// GET /api/definitions/affinities - List all affinity definitions
pub async fn list_affinities(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::definitions::AffinityDefinition>>, ApiError> {
    let affs = state.definitions.list_all_affinities();
    Ok(Json(affs.into_iter().cloned().collect()))
}

/// GET /api/definitions/effects - List all effect definitions
pub async fn list_effects(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::definitions::EffectDefinition>>, ApiError> {
    let effects = state.definitions.list_all_effects();
    Ok(Json(effects.into_iter().cloned().collect()))
}

// ============================================================================
// ATTRIBUTE DEFINITION CRUD HANDLERS (New)
// ============================================================================

#[derive(Deserialize)]
pub struct CreateAttributeRequest {
    pub name: String,
    pub description: String,
    pub category: crate::models::definitions::AttributeCategory,
    pub base_value: u32,
    pub min_value: u32,
    pub max_value: u32,
    pub training_difficulty: Percentage,
    pub icon: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateAttributeRequest {
    pub name: String,
    pub description: String,
    pub category: crate::models::definitions::AttributeCategory,
    pub base_value: u32,
    pub min_value: u32,
    pub max_value: u32,
    pub training_difficulty: Percentage,
    pub icon: Option<String>,
}

/// POST /api/definitions/attributes - Create new attribute definition
/// API handler is a THIN WRAPPER ONLY - no business logic
pub async fn create_attribute(
    State(state): State<AppState>,
    Json(req): Json<CreateAttributeRequest>,
) -> Result<(StatusCode, Json<crate::models::definitions::AttributeDefinition>), ApiError> {
    // 1. Build attribute definition
    let attr = crate::models::definitions::AttributeDefinition {
        id: AttributeId::new(),
        name: req.name,
        description: req.description,
        category: req.category,
        base_value: req.base_value,
        min_value: req.min_value,
        max_value: req.max_value,
        training_difficulty: req.training_difficulty,
        icon: req.icon,
    };
    
    // 2. Call service (ALL business logic here)
    let created = state.attribute_service
        .create_attribute(attr)
        .await
        .map_err(ApiError::from_business_error)?;
    
    // 3. Return response
    Ok((StatusCode::CREATED, Json(created)))
}

/// GET /api/definitions/attributes/:id - Get single attribute definition
/// API handler is a THIN WRAPPER ONLY - no business logic
pub async fn get_attribute(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<crate::models::definitions::AttributeDefinition>, ApiError> {
    // 1. Parse ID
    let uuid = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidInput("Invalid attribute ID format".to_string()))?;
    let attr_id = AttributeId::from(uuid);
    
    // 2. Call service (ALL business logic here)
    let attr = state.attribute_service
        .get_attribute(attr_id)
        .await
        .map_err(ApiError::from_business_error)?;
    
    // 3. Return response
    Ok(Json(attr))
}

/// PUT /api/definitions/attributes/:id - Update attribute definition
/// API handler is a THIN WRAPPER ONLY - no business logic
pub async fn update_attribute(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(req): Json<UpdateAttributeRequest>,
) -> Result<Json<crate::models::definitions::AttributeDefinition>, ApiError> {
    // 1. Parse ID
    let uuid = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidInput("Invalid attribute ID format".to_string()))?;
    let attr_id = AttributeId::from(uuid);
    
    // 2. Build attribute definition
    let attr = crate::models::definitions::AttributeDefinition {
        id: attr_id,
        name: req.name,
        description: req.description,
        category: req.category,
        base_value: req.base_value,
        min_value: req.min_value,
        max_value: req.max_value,
        training_difficulty: req.training_difficulty,
        icon: req.icon,
    };
    
    // 3. Call service (ALL business logic here)
    let updated = state.attribute_service
        .update_attribute(attr_id, attr)
        .await
        .map_err(ApiError::from_business_error)?;
    
    // 4. Return response
    Ok(Json(updated))
}

/// DELETE /api/definitions/attributes/:id - Delete attribute definition
/// API handler is a THIN WRAPPER ONLY - no business logic
pub async fn delete_attribute(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<StatusCode, ApiError> {
    // 1. Parse ID
    let uuid = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidInput("Invalid attribute ID format".to_string()))?;
    let attr_id = AttributeId::from(uuid);
    
    // 2. Call service (ALL business logic here)
    state.attribute_service
        .delete_attribute(attr_id)
        .await
        .map_err(ApiError::from_business_error)?;
    
    // 3. Return response
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/definitions/attributes?q=search - Search/list attributes
/// API handler is a THIN WRAPPER ONLY - no business logic
pub async fn search_attributes(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<crate::models::definitions::AttributeDefinition>>, ApiError> {
    // 1. Call service (ALL business logic here)
    let attrs = if let Some(query) = params.get("q") {
        state.attribute_service.search(query).await
    } else {
        state.attribute_service.list_all().await
    }
    .map_err(ApiError::from_business_error)?;
    
    // 2. Return response
    Ok(Json(attrs))
}

// ============================================================================
// HEALTH CHECK
// ============================================================================

/// GET /health - Health check endpoint
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug)]
pub enum ApiError {
    NoActiveGame,
    InvalidInput(String),
    Conflict(String),
    StorageError(String),
    GameError(String),
    NotFound,
    BusinessError(String),
}

impl ApiError {
    /// Convert from business error (NO business logic)
    fn from_business_error(err: crate::business::BusinessError) -> Self {
        match err {
            crate::business::BusinessError::NotFound => ApiError::NotFound,
            crate::business::BusinessError::DuplicateName(name) => 
                ApiError::Conflict(format!("Duplicate name: {}", name)),
            crate::business::BusinessError::ValidationError(msg) => 
                ApiError::InvalidInput(msg),
            crate::business::BusinessError::IdMismatch { expected, actual } => 
                ApiError::InvalidInput(format!("ID mismatch: expected {}, got {}", expected, actual)),
            crate::business::BusinessError::PersistenceError(msg) => 
                ApiError::StorageError(msg),
            crate::business::BusinessError::BusinessRuleViolation(msg) => 
                ApiError::BusinessError(msg),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::NoActiveGame => (StatusCode::NOT_FOUND, "No active game session".to_string()),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
            ApiError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            ApiError::StorageError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::GameError(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::BusinessError(msg) => (StatusCode::BAD_REQUEST, msg),
        };
        
        (status, message).into_response()
    }
}
