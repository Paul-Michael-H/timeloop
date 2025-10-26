// Systems - Bevy systems for state management and network communication

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use bevy::prelude::*;
use crate::client::state::{
    GameState, UiState, ApiClientResource, PollTimer, ConnectionStatus
};
use crate::client::events::*;

/// System to poll the server for current game state
pub fn poll_game_state(
    time: Res<Time>,
    mut poll_timer: ResMut<PollTimer>,
    game_state: Res<GameState>,
    api_client: Res<ApiClientResource>,
    mut event_writer: EventWriter<GameStateUpdated>,
    mut error_writer: EventWriter<ApiErrorOccurred>,
) {
    // Only poll if we have an active game loaded
    if !game_state.loaded {
        return;
    }

    // Tick the timer
    if !poll_timer.timer.tick(time.delta()).just_finished() {
        return;
    }

    // Create a channel for async communication
    let (tx, rx) = std::sync::mpsc::channel();
    let client = api_client.client.clone();
    
    // Spawn async task to fetch game state
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            client.get_current_game().await
        });
        let _ = tx.send(result);
    });

    // Check for results (non-blocking)
    if let Ok(result) = rx.try_recv() {
        match result {
            Ok(response) => {
                event_writer.send(GameStateUpdated { response });
            }
            Err(e) => {
                error_writer.send(ApiErrorOccurred {
                    error: format!("Failed to fetch game state: {:?}", e),
                });
            }
        }
    }
}

/// System to handle game state update events
pub fn handle_game_state_updated(
    mut game_state: ResMut<GameState>,
    mut event_reader: EventReader<GameStateUpdated>,
) {
    for event in event_reader.read() {
        *game_state = GameState::from_response(event.response.clone());
        info!("Game state updated: {} (Loop {}, Tick {})",
            game_state.character_name.as_ref().unwrap_or(&"Unknown".to_string()),
            game_state.current_loop,
            game_state.current_tick
        );
    }
}

/// System to handle API errors
pub fn handle_api_errors(
    mut ui_state: ResMut<UiState>,
    mut event_reader: EventReader<ApiErrorOccurred>,
) {
    for event in event_reader.read() {
        error!("API Error: {}", event.error);
        ui_state.status_message = event.error.clone();
        ui_state.connection_status = ConnectionStatus::Error;
    }
}

/// Startup system to perform initial health check
pub fn startup_health_check(
    api_client: Res<ApiClientResource>,
    mut event_writer: EventWriter<ConnectionStatusChanged>,
) {
    info!("Performing startup health check...");
    
    let client = api_client.client.clone();
    
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            client.health_check().await
        });
        
        match result {
            Ok(true) => {
                info!("✓ Server health check: OK");
            }
            Ok(false) => {
                warn!("✗ Server health check: FAILED");
            }
            Err(e) => {
                error!("✗ Server health check error: {:?}", e);
            }
        }
    });
}

/// System to handle connection status change events
pub fn handle_connection_status_changed(
    mut ui_state: ResMut<UiState>,
    mut event_reader: EventReader<ConnectionStatusChanged>,
) {
    for event in event_reader.read() {
        if event.connected {
            ui_state.connection_status = ConnectionStatus::Connected;
            ui_state.status_message = "Connected to server".to_string();
        } else {
            ui_state.connection_status = ConnectionStatus::Disconnected;
            ui_state.status_message = "Disconnected from server".to_string();
        }
    }
}

/// System to handle create game requests
pub fn handle_create_game_request(
    mut game_state: ResMut<GameState>,
    mut ui_state: ResMut<UiState>,
    api_client: Res<ApiClientResource>,
    mut event_reader: EventReader<CreateGameRequest>,
    mut created_writer: EventWriter<GameCreated>,
    mut error_writer: EventWriter<ApiErrorOccurred>,
) {
    for event in event_reader.read() {
        info!("Creating new game: {}", event.name);
        
        let client = api_client.client.clone();
        let name = event.name.clone();
        let attrs = event.starting_attributes.clone();
        
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                client.create_new_game(name.clone(), attrs).await
            });
            
            match result {
                Ok(response) => {
                    info!("✓ Game created: {} ({})", response.name, response.character_id);
                }
                Err(e) => {
                    error!("✗ Failed to create game: {:?}", e);
                }
            }
        });
    }
}

/// System to handle advance tick requests
pub fn handle_advance_tick_request(
    game_state: Res<GameState>,
    mut ui_state: ResMut<UiState>,
    api_client: Res<ApiClientResource>,
    mut event_reader: EventReader<AdvanceTickRequest>,
    mut advanced_writer: EventWriter<TickAdvanced>,
    mut error_writer: EventWriter<ApiErrorOccurred>,
) {
    for event in event_reader.read() {
        if !game_state.loaded {
            ui_state.status_message = "No active game".to_string();
            continue;
        }

        let client = api_client.client.clone();
        let ticks = event.ticks;
        
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                client.advance_tick(ticks).await
            });
            
            match result {
                Ok(response) => {
                    info!("✓ Advanced {} ticks. Current tick: {}", ticks, response.current_tick);
                }
                Err(e) => {
                    error!("✗ Failed to advance tick: {:?}", e);
                }
            }
        });
    }
}

/// System to handle set training requests
pub fn handle_set_training_request(
    game_state: Res<GameState>,
    mut ui_state: ResMut<UiState>,
    api_client: Res<ApiClientResource>,
    mut event_reader: EventReader<SetTrainingRequest>,
    mut error_writer: EventWriter<ApiErrorOccurred>,
) {
    for event in event_reader.read() {
        if !game_state.loaded {
            ui_state.status_message = "No active game".to_string();
            continue;
        }

        let client = api_client.client.clone();
        let attr_id = event.attribute_id.clone();
        
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                client.set_training(attr_id.clone()).await
            });
            
            match result {
                Ok(response) => {
                    info!("✓ Training updated: {}", response.message);
                }
                Err(e) => {
                    error!("✗ Failed to set training: {:?}", e);
                }
            }
        });
    }
}

/// System to handle acquire affinity requests
pub fn handle_acquire_affinity_request(
    game_state: Res<GameState>,
    mut ui_state: ResMut<UiState>,
    api_client: Res<ApiClientResource>,
    mut event_reader: EventReader<AcquireAffinityRequest>,
    mut error_writer: EventWriter<ApiErrorOccurred>,
) {
    for event in event_reader.read() {
        if !game_state.loaded {
            ui_state.status_message = "No active game".to_string();
            continue;
        }

        let client = api_client.client.clone();
        let aff_id = event.affinity_id.clone();
        
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                client.acquire_affinity(aff_id.clone()).await
            });
            
            match result {
                Ok(response) => {
                    info!("✓ Affinity acquired: {}", response.message);
                }
                Err(e) => {
                    error!("✗ Failed to acquire affinity: {:?}", e);
                }
            }
        });
    }
}

/// System to handle save game requests
pub fn handle_save_game_request(
    game_state: Res<GameState>,
    mut ui_state: ResMut<UiState>,
    api_client: Res<ApiClientResource>,
    mut event_reader: EventReader<SaveGameRequest>,
    mut saved_writer: EventWriter<GameSaved>,
    mut error_writer: EventWriter<ApiErrorOccurred>,
) {
    for event in event_reader.read() {
        if !game_state.loaded {
            ui_state.status_message = "No active game".to_string();
            continue;
        }

        let client = api_client.client.clone();
        
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                client.save_game().await
            });
            
            match result {
                Ok(response) => {
                    info!("✓ Game saved: {}", response.message);
                }
                Err(e) => {
                    error!("✗ Failed to save game: {:?}", e);
                }
            }
        });
    }
}
