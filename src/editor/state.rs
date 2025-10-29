// Editor State Management - Now uses API instead of direct file I/O

use crate::models::definitions::AttributeDefinition;
use crate::models::common::{AttributeId, Percentage};
use crate::editor::api_client::EditorApiClient;
use bevy::prelude::Resource;

/// Main editor state resource
#[derive(Resource, Clone)]
pub struct EditorState {
    /// All loaded attributes (from server)
    pub attributes: Vec<AttributeDefinition>,
    
    /// Currently selected attribute (index into attributes vec)
    pub selected_index: Option<usize>,
    
    /// Attribute being edited (clone of selected, or new)
    pub editing_attribute: Option<AttributeDefinition>,
    
    /// API client for server communication
    pub api_client: EditorApiClient,
    
    /// Whether current edit has unsaved changes
    pub is_dirty: bool,
    
    /// Search/filter text
    pub search_text: String,
    
    /// Status message to display
    pub status_message: String,
    
    /// Whether to show delete confirmation dialog
    pub show_delete_confirmation: bool,
    
    /// Validation errors for current edit (from server)
    pub validation_errors: Vec<String>,
    
    /// Validation warnings for current edit (from server)
    pub validation_warnings: Vec<String>,
    
    /// Server connection status
    pub server_connected: bool,
    
    /// Pending async operation (for UI feedback)
    pub pending_operation: Option<String>,
}

impl EditorState {
    pub fn new(api_url: &str) -> Self {
        Self {
            attributes: Vec::new(),
            selected_index: None,
            editing_attribute: None,
            api_client: EditorApiClient::new(api_url),
            is_dirty: false,
            search_text: String::new(),
            status_message: "Connecting to server...".to_string(),
            show_delete_confirmation: false,
            validation_errors: Vec::new(),
            validation_warnings: Vec::new(),
            server_connected: false,
            pending_operation: None,
        }
    }
    
    /// Load attributes from server (replaces load_attributes from file)
    pub async fn refresh_from_server(&mut self) -> Result<(), String> {
        self.pending_operation = Some("Loading...".to_string());
        
        match self.api_client.list_attributes().await {
            Ok(attrs) => {
                self.attributes = attrs;
                self.server_connected = true;
                self.status_message = format!("✓ Loaded {} attributes from server", self.attributes.len());
                self.pending_operation = None;
                Ok(())
            }
            Err(e) => {
                self.server_connected = false;
                self.status_message = format!("✗ Server error: {}", e);
                self.pending_operation = None;
                Err(e.to_string())
            }
        }
    }
    
    /// Save current edit to server (replaces save_to_file)
    pub async fn save_current_to_server(&mut self) -> Result<(), String> {
        if let Some(attr) = &self.editing_attribute {
            self.pending_operation = Some("Saving...".to_string());
            
            let result = if self.selected_index.is_some() {
                // Update existing
                self.api_client.update_attribute(attr.id, attr.clone()).await
            } else {
                // Create new
                self.api_client.create_attribute(attr.clone()).await
            };
            
            match result {
                Ok(saved_attr) => {
                    // Update local list
                    if let Some(idx) = self.selected_index {
                        self.attributes[idx] = saved_attr.clone();
                    } else {
                        self.attributes.push(saved_attr.clone());
                        // Select the newly created item
                        self.selected_index = Some(self.attributes.len() - 1);
                    }
                    self.editing_attribute = Some(saved_attr);
                    self.is_dirty = false;
                    self.status_message = "✓ Saved successfully".to_string();
                    self.pending_operation = None;
                    Ok(())
                }
                Err(e) => {
                    self.status_message = format!("✗ Save failed: {}", e);
                    self.pending_operation = None;
                    Err(e.to_string())
                }
            }
        } else {
            Err("No attribute to save".to_string())
        }
    }
    
    /// Delete selected attribute from server
    pub async fn delete_selected_from_server(&mut self) -> Result<(), String> {
        if let Some(idx) = self.selected_index {
            let attr = &self.attributes[idx];
            self.pending_operation = Some("Deleting...".to_string());
            
            match self.api_client.delete_attribute(attr.id).await {
                Ok(_) => {
                    let deleted_name = self.attributes[idx].name.clone();
                    self.attributes.remove(idx);
                    self.selected_index = None;
                    self.editing_attribute = None;
                    self.is_dirty = false;
                    self.show_delete_confirmation = false;
                    self.status_message = format!("✓ Deleted '{}'", deleted_name);
                    self.pending_operation = None;
                    Ok(())
                }
                Err(e) => {
                    self.status_message = format!("✗ Delete failed: {}", e);
                    self.pending_operation = None;
                    Err(e.to_string())
                }
            }
        } else {
            Err("No attribute selected".to_string())
        }
    }
    
    /// Check server connection
    pub async fn check_server_connection(&mut self) -> bool {
        match self.api_client.health_check().await {
            Ok(true) => {
                self.server_connected = true;
                true
            }
            _ => {
                self.server_connected = false;
                false
            }
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
    
    /// Save current edit back to the list (local only - call save_current_to_server for persistence)
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
            self.status_message = "Changes saved locally (not persisted to server yet)".to_string();
        }
    }
    
    /// Revert current edit
    pub fn revert_edit(&mut self) {
        self.editing_attribute = None;
        self.is_dirty = false;
        self.status_message = "Changes reverted".to_string();
    }
    
    /// Delete currently selected attribute (local only - call delete_selected_from_server for persistence)
    pub fn delete_selected(&mut self) {
        if let Some(idx) = self.selected_index {
            let deleted_name = self.attributes[idx].name.clone();
            self.attributes.remove(idx);
            self.selected_index = None;
            self.editing_attribute = None;
            self.is_dirty = false;
            self.show_delete_confirmation = false;
            self.status_message = format!("Deleted '{}' locally (not persisted to server yet)", deleted_name);
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
        Self::new("http://127.0.0.1:3000")
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
