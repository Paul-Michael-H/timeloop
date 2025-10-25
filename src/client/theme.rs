// Cobalt Theme - VS Code Cobalt color palette for Bevy UI
// Based on Visual Studio Code's Cobalt theme

use bevy::prelude::*;

/// Cobalt theme resource containing all color definitions
#[derive(Resource)]
#[allow(dead_code)]
pub struct CobaltTheme {
    // Background colors
    pub bg_dark: Color,
    pub bg_medium: Color,
    pub bg_light: Color,
    
    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    
    // Accent colors
    pub accent_blue: Color,
    pub accent_cyan: Color,
    pub accent_orange: Color,
    pub accent_green: Color,
    pub accent_purple: Color,
    
    // Progress/Status colors
    pub progress_bg: Color,
    pub progress_fill: Color,
    pub training_indicator: Color,
}

impl Default for CobaltTheme {
    fn default() -> Self {
        Self {
            // Background colors (from VS Code Cobalt)
            bg_dark: Color::srgb(0.09, 0.11, 0.15),        // #193549
            bg_medium: Color::srgb(0.12, 0.15, 0.20),      // #1F2D3A
            bg_light: Color::srgb(0.15, 0.19, 0.25),       // #27394F
            
            // Text colors
            text_primary: Color::srgb(1.0, 1.0, 1.0),      // #FFFFFF
            text_secondary: Color::srgb(0.8, 0.8, 0.8),    // #CCCCCC
            text_muted: Color::srgb(0.5, 0.6, 0.7),        // #8A99A6
            
            // Accent colors
            accent_blue: Color::srgb(0.31, 0.59, 0.84),    // #5095D6
            accent_cyan: Color::srgb(0.0, 0.8, 0.8),       // #00CCCC
            accent_orange: Color::srgb(1.0, 0.6, 0.0),     // #FF9D00
            accent_green: Color::srgb(0.6, 0.8, 0.2),      // #99CC33
            accent_purple: Color::srgb(0.8, 0.4, 0.8),     // #CC66CC
            
            // Progress/Status colors
            progress_bg: Color::srgb(0.2, 0.25, 0.3),      // #334455
            progress_fill: Color::srgb(0.31, 0.59, 0.84),  // Same as accent_blue
            training_indicator: Color::srgb(1.0, 0.6, 0.0), // Same as accent_orange
        }
    }
}

#[allow(dead_code)]
impl CobaltTheme {
    // Constant colors for use in startup systems
    pub const BG_DARK: Color = Color::srgb(0.09, 0.11, 0.15);
    pub const BG_MEDIUM: Color = Color::srgb(0.12, 0.15, 0.20);
    pub const BG_LIGHT: Color = Color::srgb(0.15, 0.19, 0.25);
    
    pub const TEXT_PRIMARY: Color = Color::srgb(1.0, 1.0, 1.0);
    pub const TEXT_SECONDARY: Color = Color::srgb(0.8, 0.8, 0.8);
    pub const TEXT_MUTED: Color = Color::srgb(0.5, 0.6, 0.7);
    
    pub const ACCENT_BLUE: Color = Color::srgb(0.31, 0.59, 0.84);
    pub const ACCENT_CYAN: Color = Color::srgb(0.0, 0.8, 0.8);
    pub const ACCENT_ORANGE: Color = Color::srgb(1.0, 0.6, 0.0);
    pub const ACCENT_GREEN: Color = Color::srgb(0.6, 0.8, 0.2);
    pub const ACCENT_PURPLE: Color = Color::srgb(0.8, 0.4, 0.8);
    
    /// Create a button style with Cobalt theme
    pub fn button_style() -> Style {
        Style {
            padding: UiRect::all(Val::Px(12.0)),
            margin: UiRect::all(Val::Px(4.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        }
    }
    
    /// Create a panel style with Cobalt theme
    pub fn panel_style() -> Style {
        Style {
            padding: UiRect::all(Val::Px(16.0)),
            margin: UiRect::all(Val::Px(8.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        }
    }
    
    /// Create a text style with default font size
    pub fn text_style(font: Handle<Font>, size: f32, color: Color) -> TextStyle {
        TextStyle {
            font,
            font_size: size,
            color,
        }
    }
}
