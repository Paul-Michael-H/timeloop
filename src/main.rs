// Timeloop Client - Bevy UI for the Timeloop game
// A deterministic time-loop RPG with Cobalt theme

mod client;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use client::theme::CobaltTheme;
use client::state::{GameState, UiState, ApiClientResource, PollTimer};
use client::systems::*;
use client::events::*;
use client::ui::render_ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Timeloop".into(),
                resolution: (1280.0, 720.0).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .insert_resource(ClearColor(CobaltTheme::BG_DARK))
        .insert_resource(CobaltTheme::default())
        .insert_resource(GameState::default())
        .insert_resource(UiState::default())
        .insert_resource(ApiClientResource::default())
        .insert_resource(PollTimer::default())
        // Register all events
        .add_event::<GameStateUpdated>()
        .add_event::<ApiErrorOccurred>()
        .add_event::<CreateGameRequest>()
        .add_event::<AdvanceTickRequest>()
        .add_event::<SetTrainingRequest>()
        .add_event::<AcquireAffinityRequest>()
        .add_event::<SaveGameRequest>()
        .add_event::<GameCreated>()
        .add_event::<TickAdvanced>()
        .add_event::<GameSaved>()
        .add_event::<ConnectionStatusChanged>()
        // Startup systems
        .add_systems(Startup, setup)
        .add_systems(Startup, startup_health_check)
        // Update systems
        .add_systems(Update, handle_game_state_updated)
        .add_systems(Update, handle_api_errors)
        .add_systems(Update, handle_connection_status_changed)
        .add_systems(Update, handle_create_game_request)
        .add_systems(Update, handle_advance_tick_request)
        .add_systems(Update, handle_set_training_request)
        .add_systems(Update, handle_acquire_affinity_request)
        .add_systems(Update, handle_save_game_request)
        // UI system
        .add_systems(Update, render_ui)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn camera
    commands.spawn(Camera2dBundle::default());
    
    info!("Timeloop client started with Cobalt theme");
    info!("API client initialized for http://127.0.0.1:3000");
}
