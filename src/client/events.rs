// Events - Bevy events for API responses and UI actions

#![allow(dead_code)]

use bevy::prelude::*;
use crate::client::api::{GameStateResponse, AttributeDefinitionInfo};
use std::collections::HashMap;

/// Event fired when attribute definitions are loaded
#[derive(Event)]
pub struct AttributeDefinitionsLoaded {
    pub definitions: Vec<AttributeDefinitionInfo>,
}

/// Event fired when game state is successfully fetched from server
#[derive(Event)]
pub struct GameStateUpdated {
    pub response: GameStateResponse,
}

/// Event fired when an API error occurs
#[derive(Event)]
pub struct ApiErrorOccurred {
    pub error: String,
}

/// Event to request creating a new game
#[derive(Event)]
pub struct CreateGameRequest {
    pub name: String,
    pub starting_attributes: HashMap<String, u32>,
}

/// Event to request advancing game ticks
#[derive(Event)]
pub struct AdvanceTickRequest {
    pub ticks: u64,
}

/// Event to request setting training
#[derive(Event)]
pub struct SetTrainingRequest {
    pub attribute_id: Option<String>,
}

/// Event to request acquiring an affinity
#[derive(Event)]
pub struct AcquireAffinityRequest {
    pub affinity_id: String,
}

/// Event to request saving the game
#[derive(Event)]
pub struct SaveGameRequest;

/// Event fired when game is successfully created
#[derive(Event)]
pub struct GameCreated {
    pub character_id: String,
    pub name: String,
}

/// Event fired when tick is advanced
#[derive(Event)]
pub struct TickAdvanced {
    pub current_tick: u64,
    pub progress: Vec<String>,
}

/// Event fired when game is saved
#[derive(Event)]
pub struct GameSaved {
    pub message: String,
}

/// Event fired when connection status changes
#[derive(Event)]
pub struct ConnectionStatusChanged {
    pub connected: bool,
}
