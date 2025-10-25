// Timeloop Client - Bevy UI for the Timeloop game
// A deterministic time-loop RPG with Cobalt theme

mod client;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use client::theme::CobaltTheme;

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
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn camera
    commands.spawn(Camera2dBundle::default());
    
    info!("Timeloop client started with Cobalt theme");
}
