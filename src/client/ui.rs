// UI - egui interface for the Timeloop client

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::collections::HashMap;

use crate::client::theme::CobaltTheme;
use crate::client::state::{GameState, UiState, ConnectionStatus};
use crate::client::events::*;

/// Main UI system that renders the entire interface
#[allow(clippy::too_many_arguments)]
pub fn render_ui(
    mut contexts: EguiContexts,
    theme: Res<CobaltTheme>,
    game_state: Res<GameState>,
    mut ui_state: ResMut<UiState>,
    mut create_game_events: EventWriter<CreateGameRequest>,
    mut advance_tick_events: EventWriter<AdvanceTickRequest>,
    mut set_training_events: EventWriter<SetTrainingRequest>,
    mut save_game_events: EventWriter<SaveGameRequest>,
) {
    let ctx = contexts.ctx_mut();
    
    // Apply Cobalt theme to egui
    apply_cobalt_theme(ctx, &theme);
    
    // Top bar with connection status
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("⏰ Timeloop");
            ui.separator();
            
            // Connection status indicator
            let (status_text, status_color) = match ui_state.connection_status {
                ConnectionStatus::Connected => ("● Connected", egui::Color32::from_rgb(0, 255, 0)),
                ConnectionStatus::Disconnected => ("● Disconnected", egui::Color32::from_rgb(255, 0, 0)),
                ConnectionStatus::Error => ("● Error", egui::Color32::from_rgb(255, 165, 0)),
            };
            ui.colored_label(status_color, status_text);
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !ui_state.status_message.is_empty() {
                    ui.label(&ui_state.status_message);
                }
            });
        });
    });
    
    // Main content area
    egui::CentralPanel::default().show(ctx, |ui| {
        if !game_state.loaded {
            // Show main menu / new game screen
            render_main_menu(ui, &mut ui_state, &mut create_game_events);
        } else {
            // Show game dashboard
            render_game_dashboard(
                ui,
                &game_state,
                &mut ui_state,
                &mut advance_tick_events,
                &mut set_training_events,
                &mut save_game_events,
            );
        }
    });
}

/// Render the main menu / new game screen
fn render_main_menu(
    ui: &mut egui::Ui,
    ui_state: &mut UiState,
    create_game_events: &mut EventWriter<CreateGameRequest>,
) {
    ui.vertical_centered(|ui| {
        ui.add_space(50.0);
        
        ui.heading("Welcome to Timeloop");
        ui.add_space(20.0);
        ui.label("A deterministic time-loop RPG");
        ui.add_space(40.0);
        
        // New Game section
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(31, 45, 58)) // BG_MEDIUM
            .rounding(5.0)
            .inner_margin(20.0)
            .show(ui, |ui| {
                ui.heading("Create New Game");
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.label("Character Name:");
                    ui.text_edit_singleline(&mut ui_state.status_message);
                });
                
                ui.add_space(10.0);
                ui.label("Starting Attributes (Total: 20 points)");
                ui.add_space(5.0);
                
                // Attribute sliders - using current phase attributes: Physical, Mental only
                let mut physical = 10;
                let mut mental = 10;
                
                ui.horizontal(|ui| {
                    ui.label("Physical:");
                    ui.add(egui::Slider::new(&mut physical, 5..=20));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Mental:");
                    ui.add(egui::Slider::new(&mut mental, 5..=20));
                });
                
                let total = physical + mental;
                ui.add_space(5.0);
                ui.label(format!("Total: {} / 20", total));
                
                ui.add_space(15.0);
                
                if ui.button("Create Game").clicked() {
                    let name = if ui_state.status_message.is_empty() {
                        "Timelooper".to_string()
                    } else {
                        ui_state.status_message.clone()
                    };
                    
                    let mut attrs = HashMap::new();
                    attrs.insert("physical".to_string(), physical as u32);
                    attrs.insert("mental".to_string(), mental as u32);
                    
                    create_game_events.send(CreateGameRequest {
                        name,
                        starting_attributes: attrs,
                    });
                    
                    ui_state.status_message = "Creating game...".to_string();
                }
            });
    });
}

/// Render the game dashboard
fn render_game_dashboard(
    ui: &mut egui::Ui,
    game_state: &GameState,
    _ui_state: &mut UiState,
    advance_tick_events: &mut EventWriter<AdvanceTickRequest>,
    set_training_events: &mut EventWriter<SetTrainingRequest>,
    save_game_events: &mut EventWriter<SaveGameRequest>,
) {
    // Left panel - Character info and controls
    egui::SidePanel::left("left_panel")
        .resizable(false)
        .min_width(300.0)
        .show_inside(ui, |ui| {
            ui.heading("Character");
            ui.separator();
            
            if let Some(name) = &game_state.character_name {
                ui.label(format!("Name: {}", name));
            }
            
            ui.label(format!("Loop: {}", game_state.current_loop));
            ui.label(format!("Tick: {}", game_state.current_tick));
            
            ui.add_space(20.0);
            
            // Time controls
            ui.heading("Time Control");
            ui.separator();
            
            ui.horizontal(|ui| {
                if ui.button("Advance 1 Tick").clicked() {
                    advance_tick_events.send(AdvanceTickRequest { ticks: 1 });
                }
                
                if ui.button("Advance 10 Ticks").clicked() {
                    advance_tick_events.send(AdvanceTickRequest { ticks: 10 });
                }
            });
            
            ui.add_space(20.0);
            
            // Save button
            if ui.button("💾 Save Game").clicked() {
                save_game_events.send(SaveGameRequest);
            }
        });
    
    // Main content area - Attributes and Affinities
    egui::CentralPanel::default().show_inside(ui, |ui| {
        ui.heading("Attributes");
        ui.separator();
        
        // Attributes table
        egui::Grid::new("attributes_grid")
            .striped(true)
            .show(ui, |ui| {
                ui.label("Attribute");
                ui.label("Value");
                ui.label("Training");
                ui.end_row();
                
                for attr in &game_state.attributes {
                    ui.label(&attr.name);
                    ui.label(format!("{}", attr.value));
                    
                    let button_text = if attr.training { "✓ Training" } else { "Train" };
                    if ui.button(button_text).clicked() {
                        let attr_id = if attr.training {
                            None
                        } else {
                            Some(attr.id.clone())
                        };
                        
                        set_training_events.send(SetTrainingRequest {
                            attribute_id: attr_id,
                        });
                    }
                    
                    ui.end_row();
                }
            });
        
        ui.add_space(30.0);
        
        ui.heading("Affinities");
        ui.separator();
        
        if game_state.affinities.is_empty() {
            ui.label("No affinities acquired yet.");
        } else {
            // Affinities table
            egui::Grid::new("affinities_grid")
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Affinity");
                    ui.label("Mastery Level");
                    ui.end_row();
                    
                    for aff in &game_state.affinities {
                        ui.label(&aff.name);
                        ui.label(format!("{}", aff.mastery_level));
                        ui.end_row();
                    }
                });
        }
    });
}

/// Apply Cobalt theme colors to egui context
fn apply_cobalt_theme(ctx: &egui::Context, theme: &CobaltTheme) {
    let mut style = (*ctx.style()).clone();
    let visuals = &mut style.visuals;
    
    // Convert Bevy colors to egui colors
    let bg_dark = color_to_egui(theme.bg_dark);
    let bg_medium = color_to_egui(theme.bg_medium);
    let text = color_to_egui(theme.text_primary);
    let text_dim = color_to_egui(theme.text_muted);
    let accent = color_to_egui(theme.accent_blue);
    
    // Apply colors
    visuals.panel_fill = bg_dark;
    visuals.window_fill = bg_medium;
    visuals.extreme_bg_color = bg_dark;
    visuals.faint_bg_color = bg_medium;
    
    visuals.widgets.noninteractive.bg_fill = bg_medium;
    visuals.widgets.noninteractive.fg_stroke.color = text;
    
    visuals.widgets.inactive.bg_fill = bg_medium;
    visuals.widgets.inactive.fg_stroke.color = text_dim;
    
    visuals.widgets.hovered.bg_fill = color_to_egui(theme.bg_light);
    visuals.widgets.hovered.fg_stroke.color = text;
    
    visuals.widgets.active.bg_fill = accent;
    visuals.widgets.active.fg_stroke.color = text;
    
    visuals.selection.bg_fill = accent;
    visuals.selection.stroke.color = text;
    
    ctx.set_style(style);
}

/// Convert Bevy Color to egui Color32
fn color_to_egui(color: Color) -> egui::Color32 {
    let [r, g, b, a] = color.to_srgba().to_u8_array();
    egui::Color32::from_rgba_unmultiplied(r, g, b, a)
}
