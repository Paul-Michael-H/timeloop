// Timeloop Game Object Editor
// Phase 1A: Attributes Editor

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Timeloop - Game Object Editor".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, timeloop::editor::ui_system)
        .run();
}

fn setup(mut commands: Commands) {
    // Initialize editor state
    let mut state = timeloop::editor::EditorState::new();
    
    // Try to load attributes at startup
    match timeloop::editor::io::load_attributes() {
        Ok(attrs) => {
            state.attributes = attrs;
            state.status_message = format!("Loaded {} attributes", state.attributes.len());
        }
        Err(e) => {
            state.status_message = format!("Error loading attributes: {}", e);
        }
    }
    
    commands.insert_resource(state);
    commands.insert_resource(timeloop::client::theme::CobaltTheme::default());
    
    info!("Timeloop Editor started");
}
