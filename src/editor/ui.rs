// Editor UI System - Main interface with API Communication

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::client::theme::CobaltTheme;
use crate::editor::{EditorState, validation};
use crate::models::definitions::AttributeCategory;

/// Main UI system that renders the editor interface
pub fn ui_system(
    mut contexts: EguiContexts,
    theme: Res<CobaltTheme>,
    mut state: ResMut<EditorState>,
) {
    let ctx = contexts.ctx_mut();
    
    // Apply Cobalt theme
    apply_cobalt_theme(ctx, &theme);
    
    // Top menu bar
    render_top_bar(ctx, &mut state);
    
    // Main content area
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal_top(|ui| {
            // Left panel - List view
            ui.vertical(|ui| {
                ui.set_width(350.0);
                render_list_view(ui, &mut state);
            });
            
            ui.separator();
            
            // Right panel - Detail view (take remaining space)
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true), |ui| {
                render_detail_view(ui, &mut state);
            });
        });
    });
    
    // Delete confirmation dialog
    if state.show_delete_confirmation {
        render_delete_dialog(ctx, &mut state);
    }
}

/// Render top menu bar
fn render_top_bar(ctx: &egui::Context, state: &mut EditorState) {
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("⚙ Timeloop - Attribute Editor");
            ui.separator();
            
            // Server connection indicator
            let connection_color = if state.server_connected {
                egui::Color32::from_rgb(0, 255, 0)
            } else {
                egui::Color32::from_rgb(255, 0, 0)
            };
            ui.colored_label(connection_color, if state.server_connected { "● Connected" } else { "● Disconnected" });
            ui.separator();
            
            // Load from Server button
            if ui.button("📂 Load from Server").clicked() {
                state.status_message = "Loading from server...".to_string();
                // Execute synchronously using blocking runtime
                let rt = tokio::runtime::Runtime::new().unwrap();
                match rt.block_on(state.refresh_from_server()) {
                    Ok(_) => {
                        // Status message is set by refresh_from_server
                    }
                    Err(e) => {
                        state.status_message = format!("✗ Failed to load: {}", e);
                        state.server_connected = false;
                    }
                }
            }
            
            // Save button (saves current editing attribute only)
            let can_save = state.editing_attribute.is_some() && state.validation_errors.is_empty();
            if ui.add_enabled(can_save, egui::Button::new("💾 Save to Server")).clicked() {
                state.status_message = "Saving to server...".to_string();
                // Execute synchronously using blocking runtime
                let rt = tokio::runtime::Runtime::new().unwrap();
                match rt.block_on(state.save_current_to_server()) {
                    Ok(_) => {
                        // Status message is set by save_current_to_server
                    }
                    Err(e) => {
                        state.status_message = format!("✗ Failed to save: {}", e);
                    }
                }
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Status message
                if !state.status_message.is_empty() {
                    let color = if state.status_message.starts_with("✓") {
                        egui::Color32::from_rgb(0, 255, 0)
                    } else if state.status_message.starts_with("✗") {
                        egui::Color32::from_rgb(255, 0, 0)
                    } else {
                        egui::Color32::WHITE
                    };
                    ui.colored_label(color, &state.status_message);
                }
                
                // Pending operation indicator
                if let Some(op) = &state.pending_operation {
                    ui.spinner();
                    ui.label(op);
                }
                
                // Dirty indicator
                if state.is_dirty {
                    ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "● Unsaved");
                }
            });
        });
    });
}

/// Render list view panel
fn render_list_view(ui: &mut egui::Ui, state: &mut EditorState) {
    ui.heading("Attributes");
    ui.add_space(5.0);
    
    // Search box
    ui.horizontal(|ui| {
        ui.label("🔍");
        ui.text_edit_singleline(&mut state.search_text);
    });
    ui.add_space(5.0);
    
    // New button
    if ui.button("➕ New Attribute").clicked() {
        state.start_creating();
        validation::update_validation(state);
    }
    
    ui.separator();
    ui.add_space(5.0);
    
    // List of attributes
    egui::ScrollArea::vertical()
        .id_source("attribute_list_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
        let filtered: Vec<(usize, String, String, u32)> = state.filtered_attributes()
            .into_iter()
            .map(|(idx, attr)| (idx, attr.name.clone(), format!("{:?}", attr.category), attr.base_value))
            .collect();
        
        let selected_idx = state.selected_index;
        
        if filtered.is_empty() {
            ui.label("No attributes found");
        } else {
            for (idx, name, category, base_value) in filtered {
                let is_selected = selected_idx == Some(idx);
                
                let response = ui.selectable_label(
                    is_selected,
                    format!("📊 {}", name)
                );
                
                if response.clicked() {
                    state.selected_index = Some(idx);
                    state.start_editing();
                    validation::update_validation(state);
                }
                
                // Show category and base value
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.label(format!("{} | Base: {}", category, base_value));
                });
                
                ui.add_space(3.0);
            }
        }
    });
}

/// Render detail/edit view panel
fn render_detail_view(ui: &mut egui::Ui, state: &mut EditorState) {
    if state.editing_attribute.is_none() {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label("Select an attribute to edit");
            ui.label("or create a new one");
        });
        return;
    }
    
    // Clone the attribute to avoid borrow checker issues
    let mut attr = state.editing_attribute.as_ref().unwrap().clone();
    let is_new = state.selected_index.is_none();
    let mut changed = false;
    let mut should_apply = false;
    let mut should_revert = false;
    let mut should_delete = false;
    
    ui.heading(if !is_new {
        format!("Editing: {}", attr.name)
    } else {
        "New Attribute".to_string()
    });
    
    ui.separator();
    ui.add_space(10.0);
    
    egui::ScrollArea::vertical()
        .id_source("attribute_detail_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
        // ID (read-only)
        ui.horizontal(|ui| {
            ui.label("ID:");
            let id_str = format!("{:?}", attr.id);
            ui.add_enabled(false, egui::TextEdit::singleline(&mut id_str.as_str()));
        });
        ui.add_space(5.0);
        
        // Name
        ui.horizontal(|ui| {
            ui.label("Name:").on_hover_text("Unique name for this attribute");
            let text_edit = egui::TextEdit::singleline(&mut attr.name);
            let response = ui.add(text_edit);
            if response.gained_focus() {
                if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), response.id) {
                    let ccursor = egui::text::CCursor::new(0);
                    let end = egui::text::CCursor::new(attr.name.len());
                    state.cursor.set_char_range(Some(egui::text::CCursorRange::two(ccursor, end)));
                    state.store(ui.ctx(), response.id);
                }
            }
            if response.changed() {
                changed = true;
            }
        });
        ui.add_space(5.0);
        
        // Description
        ui.horizontal(|ui| {
            ui.label("Description:").on_hover_text("What this attribute represents");
        });
        let response = ui.add(egui::TextEdit::multiline(&mut attr.description).desired_rows(3));
        if response.gained_focus() {
            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), response.id) {
                let ccursor = egui::text::CCursor::new(0);
                let end = egui::text::CCursor::new(attr.description.len());
                state.cursor.set_char_range(Some(egui::text::CCursorRange::two(ccursor, end)));
                state.store(ui.ctx(), response.id);
            }
        }
        if response.changed() {
            changed = true;
        }
        ui.add_space(5.0);
        
        // Category
        ui.horizontal(|ui| {
            ui.label("Category:").on_hover_text("Attribute category");
            let mut category_idx = match attr.category {
                AttributeCategory::Physical => 0,
                AttributeCategory::Mental => 1,
                AttributeCategory::Social => 2,
            };
            
            if egui::ComboBox::from_id_source("category")
                .selected_text(format!("{:?}", attr.category))
                .show_index(ui, &mut category_idx, 3, |i| {
                    match i {
                        0 => "Physical",
                        1 => "Mental",
                        2 => "Social",
                        _ => "Unknown",
                    }
                })
                .changed()
            {
                attr.category = match category_idx {
                    0 => AttributeCategory::Physical,
                    1 => AttributeCategory::Mental,
                    2 => AttributeCategory::Social,
                    _ => AttributeCategory::Physical,
                };
                changed = true;
            }
        });
        ui.add_space(5.0);
        
        // Base Value
        ui.horizontal(|ui| {
            ui.label("Base Value:").on_hover_text("Starting value for new characters");
            if ui.add(egui::DragValue::new(&mut attr.base_value).speed(1.0)).changed() {
                changed = true;
            }
        });
        ui.add_space(5.0);
        
        // Min Value
        ui.horizontal(|ui| {
            ui.label("Min Value:").on_hover_text("Minimum possible value");
            if ui.add(egui::DragValue::new(&mut attr.min_value).speed(1.0)).changed() {
                changed = true;
            }
        });
        ui.add_space(5.0);
        
        // Max Value
        ui.horizontal(|ui| {
            ui.label("Max Value:").on_hover_text("Maximum possible value");
            if ui.add(egui::DragValue::new(&mut attr.max_value).speed(1.0)).changed() {
                changed = true;
            }
        });
        ui.add_space(5.0);
        
        // Training Difficulty
        ui.horizontal(|ui| {
            ui.label("Training Difficulty:").on_hover_text("Percentage (100 = normal, 200 = twice as hard)");
            let mut diff_value = attr.training_difficulty.get();
            if ui.add(egui::DragValue::new(&mut diff_value).speed(1.0)).changed() {
                attr.training_difficulty = crate::models::common::Percentage::new(diff_value);
                changed = true;
            }
            ui.label("%");
        });
        ui.add_space(5.0);
        
        // Icon
        ui.horizontal(|ui| {
            ui.label("Icon:").on_hover_text("Icon filename");
            let mut icon_text = attr.icon.clone().unwrap_or_default();
            let response = ui.add(egui::TextEdit::singleline(&mut icon_text));
            if response.gained_focus() {
                if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), response.id) {
                    let ccursor = egui::text::CCursor::new(0);
                    let end = egui::text::CCursor::new(icon_text.len());
                    state.cursor.set_char_range(Some(egui::text::CCursorRange::two(ccursor, end)));
                    state.store(ui.ctx(), response.id);
                }
            }
            if response.changed() {
                attr.icon = if icon_text.is_empty() { None } else { Some(icon_text) };
                changed = true;
            }
        });
        ui.add_space(15.0);
        
        // Update state with changes
        if changed {
            state.editing_attribute = Some(attr.clone());
            state.mark_dirty();
            validation::update_validation(state);
        }
        
        // Validation messages
        render_validation_messages(ui, state);
        
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
        
        // Action buttons
        ui.horizontal(|ui| {
            // Apply/Save button
            let can_apply = state.validation_errors.is_empty();
            if ui.add_enabled(can_apply, egui::Button::new("✓ Apply Changes")).clicked() {
                should_apply = true;
            }
            
            // Revert button
            if ui.button("↶ Revert").clicked() {
                should_revert = true;
            }
            
            // Delete button (only for existing attributes)
            if !is_new && ui.button("🗑 Delete").clicked() {
                should_delete = true;
            }
        });
    });
    
    // Process actions outside the scroll area to avoid borrow issues
    if should_apply {
        state.save_current_edit();
    }
    if should_revert {
        if state.selected_index.is_some() {
            state.start_editing();
        } else {
            state.revert_edit();
        }
        validation::update_validation(state);
    }
    if should_delete {
        state.show_delete_confirmation = true;
    }
}

/// Render validation error/warning messages
fn render_validation_messages(ui: &mut egui::Ui, state: &EditorState) {
    if !state.validation_errors.is_empty() || !state.validation_warnings.is_empty() {
        ui.separator();
        ui.add_space(5.0);
        
        // Errors
        for error in &state.validation_errors {
            ui.colored_label(egui::Color32::from_rgb(255, 100, 100), format!("❌ {}", error));
        }
        
        // Warnings
        for warning in &state.validation_warnings {
            ui.colored_label(egui::Color32::from_rgb(255, 165, 0), format!("⚠ {}", warning));
        }
        
        ui.add_space(5.0);
    }
}

/// Render delete confirmation dialog
fn render_delete_dialog(ctx: &egui::Context, state: &mut EditorState) {
    egui::Window::new("Confirm Delete")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            if let Some(attr) = state.selected_attribute() {
                ui.label(format!("Are you sure you want to delete '{}'?", attr.name));
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    if ui.button("Yes, Delete").clicked() {
                        state.delete_selected();
                    }
                    
                    if ui.button("Cancel").clicked() {
                        state.show_delete_confirmation = false;
                    }
                });
            }
        });
}

/// Apply Cobalt theme to egui context
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
