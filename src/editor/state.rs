// Editor State Management

use crate::models::definitions::AttributeDefinition;
use crate::models::common::{AttributeId, Percentage};
use bevy::prelude::Resource;

/// Main editor state resource
#[derive(Debug, Resource)]
pub struct EditorState {
    /// All loaded attributes
    pub attributes: Vec<AttributeDefinition>,
    
    /// Currently selected attribute (index into attributes vec)
    pub selected_index: Option<usize>,
    
    /// Attribute being edited (clone of selected, or new)
    pub editing_attribute: Option<AttributeDefinition>,
    
    /// Whether current edit has unsaved changes
    pub is_dirty: bool,
    
    /// Search/filter text
    pub search_text: String,
    
    /// Status message to display
    pub status_message: String,
    
    /// Whether to show delete confirmation dialog
    pub show_delete_confirmation: bool,
    
    /// Validation errors for current edit
    pub validation_errors: Vec<String>,
    
    /// Validation warnings for current edit
    pub validation_warnings: Vec<String>,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            attributes: Vec::new(),
            selected_index: None,
            editing_attribute: None,
            is_dirty: false,
            search_text: String::new(),
            status_message: "Ready".to_string(),
            show_delete_confirmation: false,
            validation_errors: Vec::new(),
            validation_warnings: Vec::new(),
        }
    }
    
    /// Get the currently selected attribute
    pub fn selected_attribute(&self) -> Option<&AttributeDefinition> {
        self.selected_index.and_then(|idx| self.attributes.get(idx))
    }
    
    /// Start editing the selected attribute
    pub fn start_editing(&mut self) {
        if let Some(attr) = self.selected_attribute() {
            self.editing_attribute = Some(attr.clone());
            self.is_dirty = false;
        }
    }
    
    /// Start creating a new attribute
    pub fn start_creating(&mut self) {
        // Find a unique name by checking existing attributes
        let mut counter = 1;
        let mut name = format!("New Attribute {}", counter);
        while self.attributes.iter().any(|a| a.name == name) {
            counter += 1;
            name = format!("New Attribute {}", counter);
        }
        
        let mut new_attr = AttributeDefinition::default_new();
        new_attr.name = name;
        
        self.editing_attribute = Some(new_attr);
        self.selected_index = None;
        self.is_dirty = true;
    }
    
    /// Mark as dirty (unsaved changes)
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }
    
    /// Save current edit back to the list
    pub fn save_current_edit(&mut self) {
        if let Some(attr) = self.editing_attribute.take() {
            if let Some(idx) = self.selected_index {
                // Update existing
                self.attributes[idx] = attr;
            } else {
                // Add new
                self.attributes.push(attr);
                self.selected_index = Some(self.attributes.len() - 1);
            }
            self.is_dirty = false;
            self.status_message = "Changes saved to memory (use Save All to persist)".to_string();
        }
    }
    
    /// Revert current edit
    pub fn revert_edit(&mut self) {
        self.editing_attribute = None;
        self.is_dirty = false;
        self.status_message = "Changes reverted".to_string();
    }
    
    /// Delete currently selected attribute
    pub fn delete_selected(&mut self) {
        if let Some(idx) = self.selected_index {
            let deleted_name = self.attributes[idx].name.clone();
            self.attributes.remove(idx);
            self.selected_index = None;
            self.editing_attribute = None;
            self.is_dirty = false;
            self.show_delete_confirmation = false;
            self.status_message = format!("Deleted '{}' (use Save All to persist)", deleted_name);
        }
    }
    
    /// Get filtered attributes based on search
    pub fn filtered_attributes(&self) -> Vec<(usize, &AttributeDefinition)> {
        self.attributes
            .iter()
            .enumerate()
            .filter(|(_, attr)| {
                if self.search_text.is_empty() {
                    true
                } else {
                    attr.name.to_lowercase().contains(&self.search_text.to_lowercase())
                }
            })
            .collect()
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}

// Extension trait for AttributeDefinition
pub trait AttributeDefinitionExt {
    fn default_new() -> AttributeDefinition;
}

impl AttributeDefinitionExt for AttributeDefinition {
    fn default_new() -> AttributeDefinition {
        use crate::models::definitions::AttributeCategory;
        
        AttributeDefinition {
            id: AttributeId::new(),
            name: "New Attribute".to_string(), // Will be made unique by start_creating
            description: "A new attribute for the game".to_string(),
            category: AttributeCategory::Physical,
            base_value: 10,
            min_value: 1,
            max_value: 100,
            training_difficulty: Percentage::new(100),
            icon: None,
        }
    }
}
