// Timeloop Game Object Editor
// Phase 2: Attributes Editor with API Communication

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
        .add_systems(Update, (
            check_initial_load_system,
            timeloop::editor::ui_system,
        ))
        .run();
}

#[derive(Resource)]
struct InitialLoadState {
    attempted: bool,
}

fn setup(mut commands: Commands) {
    // Initialize editor state with API URL
    let api_url = std::env::var("TIMELOOP_API_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
    
    let state = timeloop::editor::EditorState::new(&api_url);
    
    commands.insert_resource(state);
    commands.insert_resource(InitialLoadState { attempted: false });
    commands.insert_resource(timeloop::client::theme::CobaltTheme::default());
    
    info!("Timeloop Editor started - connecting to API: {}", api_url);
}

/// System to perform initial load from server
fn check_initial_load_system(
    mut state: ResMut<timeloop::editor::EditorState>,
    mut load_state: ResMut<InitialLoadState>,
) {
    if !load_state.attempted {
        load_state.attempted = true;
        
        // Spawn async task to load from server
        let api_client = state.api_client.clone();
        
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                match api_client.health_check().await {
                    Ok(true) => {
                        info!("✓ Server is reachable");
                    }
                    _ => {
                        warn!("✗ Server is not reachable - editor will work in offline mode");
                    }
                }
            });
        });
        
        // Set initial status
        state.status_message = "Checking server connection...".to_string();
    }
}
