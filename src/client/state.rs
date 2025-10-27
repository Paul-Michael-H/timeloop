// State management - Bevy resources and game state

#![allow(dead_code)]

use bevy::prelude::*;
use crate::client::api::{ApiClient, GameStateResponse, AttributeInfo, AffinityInfo, AttributeDefinitionInfo};

/// Available attribute definitions from the server
#[derive(Resource, Default, Clone)]
pub struct AttributeDefinitions {
    pub definitions: Vec<AttributeDefinitionInfo>,
    pub loaded: bool,
}

/// Main game state resource synced from the server
#[derive(Resource, Default)]
pub struct GameState {
    pub loaded: bool,
    pub character_id: Option<String>,
    pub character_name: Option<String>,
    pub current_loop: u32,
    pub current_tick: u64,
    pub attributes: Vec<AttributeDisplay>,
    pub affinities: Vec<AffinityDisplay>,
    pub training_attribute_id: Option<String>,
}

impl GameState {
    pub fn from_response(response: GameStateResponse) -> Self {
        Self {
            loaded: true,
            character_id: Some(response.character_id),
            character_name: Some(response.name),
            current_loop: response.current_loop,
            current_tick: response.current_tick,
            attributes: response.attributes.into_iter().map(AttributeDisplay::from).collect(),
            affinities: response.affinities.into_iter().map(AffinityDisplay::from).collect(),
            training_attribute_id: None,
        }
    }
    
    pub fn clear(&mut self) {
        self.loaded = false;
        self.character_id = None;
        self.character_name = None;
        self.current_loop = 0;
        self.current_tick = 0;
        self.attributes.clear();
        self.affinities.clear();
        self.training_attribute_id = None;
    }
}

/// Display information for an attribute
#[derive(Clone, Debug)]
pub struct AttributeDisplay {
    pub id: String,
    pub name: String,
    pub value: u32,
    pub training: bool,
}

impl From<AttributeInfo> for AttributeDisplay {
    fn from(info: AttributeInfo) -> Self {
        Self {
            id: info.id,
            name: info.name,
            value: info.value,
            training: info.training,
        }
    }
}

/// Display information for an affinity
#[derive(Clone, Debug)]
pub struct AffinityDisplay {
    pub id: String,
    pub name: String,
    pub mastery_level: u32,
}

impl From<AffinityInfo> for AffinityDisplay {
    fn from(info: AffinityInfo) -> Self {
        Self {
            id: info.id,
            name: info.name,
            mastery_level: info.mastery_level,
        }
    }
}

/// UI state for selections and temporary data
#[derive(Resource, Default)]
pub struct UiState {
    pub status_message: String,
    pub selected_attribute_id: Option<String>,
    pub selected_affinity_id: Option<String>,
    pub show_affinity_modal: bool,
    pub connection_status: ConnectionStatus,
    pub character_creation_attrs: std::collections::HashMap<String, i32>,
}

#[derive(Default, PartialEq)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connected,
    Error,
}

/// API client resource
#[derive(Resource, Default)]
pub struct ApiClientResource {
    pub client: ApiClient,
}

/// Timer for periodic game state polling
#[derive(Resource)]
pub struct PollTimer {
    pub timer: Timer,
}

impl Default for PollTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        }
    }
}
