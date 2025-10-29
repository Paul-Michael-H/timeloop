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
            periodic_connection_check_system,
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
        
        // Check server connection synchronously
        let rt = tokio::runtime::Runtime::new().unwrap();
        match rt.block_on(state.check_server_connection()) {
            true => {
                info!("✓ Server is reachable - attempting to load attributes");
                match rt.block_on(state.refresh_from_server()) {
                    Ok(_) => {
                        info!("✓ Loaded {} attributes from server", state.attributes.len());
                    }
                    Err(e) => {
                        warn!("✗ Failed to load attributes: {}", e);
                    }
                }
            }
            false => {
                warn!("✗ Server is not reachable - check that timeloop-server is running");
                state.status_message = "✗ Server not reachable - is timeloop-server running?".to_string();
            }
        }
    }
}

/// System to periodically check server connection
fn periodic_connection_check_system(
    state: Res<timeloop::editor::EditorState>,
    time: Res<Time>,
) {
    // Check every 5 seconds
    static mut LAST_CHECK: f32 = 0.0;
    unsafe {
        LAST_CHECK += time.delta_seconds();
        if LAST_CHECK >= 5.0 {
            LAST_CHECK = 0.0;
            
            // Quick health check (don't block)
            let api_client = state.api_client.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let _ = rt.block_on(api_client.health_check());
            });
        }
    }
}
