// API Client - HTTP wrapper for communicating with the Timeloop server
// Handles all REST API calls to http://127.0.0.1:3000

#![allow(dead_code)]

use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// API client for communicating with the Timeloop server
#[allow(dead_code)]
#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    client: reqwest::Client,
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new("http://127.0.0.1:3000")
    }
}

#[allow(dead_code)]
impl ApiClient {
    /// Create a new API client with the specified base URL
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
        }
    }
    
    /// Check server health
    pub async fn health_check(&self) -> Result<bool, ApiError> {
        let url = format!("{}/health", self.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;
        
        Ok(response.status().is_success())
    }
    
    /// Create a new game with a character
    pub async fn create_new_game(
        &self,
        name: String,
        starting_attributes: HashMap<String, u32>,
    ) -> Result<GameStateResponse, ApiError> {
        let url = format!("{}/api/game/new", self.base_url);
        let request_body = NewGameRequest {
            name,
            starting_attributes,
        };
        
        let response = self.client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ApiError::ServerError(response.status().as_u16()));
        }
        
        response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))
    }
    
    /// Get the current game state
    pub async fn get_current_game(&self) -> Result<GameStateResponse, ApiError> {
        let url = format!("{}/api/game/current", self.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;
        
        if response.status().as_u16() == 404 {
            return Err(ApiError::NoActiveGame);
        }
        
        if !response.status().is_success() {
            return Err(ApiError::ServerError(response.status().as_u16()));
        }
        
        response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))
    }
    
    /// Advance the game by a specified number of ticks
    pub async fn advance_tick(&self, ticks: u64) -> Result<TickResponse, ApiError> {
        let url = format!("{}/api/game/tick", self.base_url);
        let request_body = AdvanceTickRequest { ticks };
        
        let response = self.client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;
        
        if response.status().as_u16() == 404 {
            return Err(ApiError::NoActiveGame);
        }
        
        if !response.status().is_success() {
            return Err(ApiError::ServerError(response.status().as_u16()));
        }
        
        response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))
    }
    
    /// Save the current game
    pub async fn save_game(&self) -> Result<SaveResponse, ApiError> {
        let url = format!("{}/api/game/save", self.base_url);
        
        let response = self.client
            .post(&url)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;
        
        if response.status().as_u16() == 404 {
            return Err(ApiError::NoActiveGame);
        }
        
        if !response.status().is_success() {
            return Err(ApiError::ServerError(response.status().as_u16()));
        }
        
        response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))
    }
    
    /// Set training for an attribute
    pub async fn set_training(
        &self,
        attribute_id: Option<String>,
    ) -> Result<StatusResponse, ApiError> {
        let url = format!("{}/api/game/train", self.base_url);
        let request_body = SetTrainingRequest { attribute_id };
        
        let response = self.client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;
        
        if response.status().as_u16() == 404 {
            return Err(ApiError::NoActiveGame);
        }
        
        if !response.status().is_success() {
            return Err(ApiError::ServerError(response.status().as_u16()));
        }
        
        response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))
    }
    
    /// Acquire a new affinity
    pub async fn acquire_affinity(&self, affinity_id: String) -> Result<StatusResponse, ApiError> {
        let url = format!("{}/api/game/acquire_affinity", self.base_url);
        let request_body = AcquireAffinityRequest { affinity_id };
        
        let response = self.client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;
        
        if response.status().as_u16() == 404 {
            return Err(ApiError::NoActiveGame);
        }
        
        if !response.status().is_success() {
            return Err(ApiError::ServerError(response.status().as_u16()));
        }
        
        response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))
    }
    
    /// List all available attributes
    pub async fn list_attributes(&self) -> Result<DefinitionsResponse, ApiError> {
        let url = format!("{}/api/definitions/attributes", self.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ApiError::ServerError(response.status().as_u16()));
        }
        
        response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))
    }
    
    /// List all attribute definitions
    pub async fn list_attribute_definitions(&self) -> Result<Vec<AttributeDefinitionInfo>, ApiError> {
        let url = format!("{}/api/definitions/attributes", self.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ApiError::ServerError(response.status().as_u16()));
        }
        
        response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))
    }
    
    /// List all available affinities
    pub async fn list_affinities(&self) -> Result<DefinitionsResponse, ApiError> {
        let url = format!("{}/api/definitions/affinities", self.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ApiError::ServerError(response.status().as_u16()));
        }
        
        response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))
    }
    
    /// List all available effects
    pub async fn list_effects(&self) -> Result<DefinitionsResponse, ApiError> {
        let url = format!("{}/api/definitions/effects", self.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ApiError::ServerError(response.status().as_u16()));
        }
        
        response
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))
    }
}

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Serialize)]
struct NewGameRequest {
    name: String,
    starting_attributes: HashMap<String, u32>,
}

#[allow(dead_code)]
#[derive(Serialize)]
struct AdvanceTickRequest {
    ticks: u64,
}

#[allow(dead_code)]
#[derive(Serialize)]
struct SetTrainingRequest {
    attribute_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Serialize)]
struct AcquireAffinityRequest {
    affinity_id: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GameStateResponse {
    pub character_id: String,
    pub name: String,
    pub current_loop: u32,
    pub current_tick: u64,
    pub attributes: Vec<AttributeInfo>,
    pub affinities: Vec<AffinityInfo>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AttributeInfo {
    pub id: String,
    pub name: String,
    pub value: u32,
    pub training: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AttributeDefinitionInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub base_value: u32,
    pub min_value: u32,
    pub max_value: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AffinityInfo {
    pub id: String,
    pub name: String,
    pub mastery_level: u32,
}

#[derive(Deserialize, Debug)]
pub struct TickResponse {
    pub current_tick: u64,
    pub progress: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct SaveResponse {
    pub status: String,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct StatusResponse {
    pub status: String,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct DefinitionsResponse {
    pub definitions: serde_json::Value,
}

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug)]
pub enum ApiError {
    NetworkError(String),
    ServerError(u16),
    ParseError(String),
    NoActiveGame,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ApiError::ServerError(code) => write!(f, "Server error: {}", code),
            ApiError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ApiError::NoActiveGame => write!(f, "No active game"),
        }
    }
}

impl std::error::Error for ApiError {}
