// Save game management
// This handles: character state serialization, save/load operations

use std::path::Path;
use crate::models::instances::Character;

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Save file not found: {0}")]
    NotFound(String),
}

// ============================================================================
// SAVE MANAGER
// ============================================================================

/// Manages character save/load operations
/// Preserves all UUIDs when loading to maintain save file consistency
pub struct SaveManager {
    save_directory: std::path::PathBuf,
}

impl SaveManager {
    /// Create a new save manager with the specified save directory
    pub fn new(save_directory: impl AsRef<Path>) -> Self {
        Self {
            save_directory: save_directory.as_ref().to_path_buf(),
        }
    }
    
    /// Save a character to disk (async version)
    pub async fn save_character(&self, character: &Character) -> Result<(), SaveError> {
        // Ensure save directory exists
        tokio::fs::create_dir_all(&self.save_directory).await?;
        
        // Serialize character to JSON
        let json = serde_json::to_string_pretty(character)?;
        
        // Write to file (using character ID as filename)
        let filename = format!("{:?}.json", character.id);
        let filepath = self.save_directory.join(filename);
        tokio::fs::write(filepath, json).await?;
        
        Ok(())
    }
    
    /// Load a character from disk (async version)
    pub async fn load_character(&self, character_id: &str) -> Result<Character, SaveError> {
        let filename = format!("{}.json", character_id);
        let filepath = self.save_directory.join(&filename);
        
        // Check if file exists
        if !filepath.exists() {
            return Err(SaveError::NotFound(filename));
        }
        
        // Read and parse JSON
        let content = tokio::fs::read_to_string(filepath).await?;
        let character = serde_json::from_str(&content)?;
        
        Ok(character)
    }
    
    /// List all saved character IDs
    pub async fn list_saved_characters(&self) -> Result<Vec<String>, SaveError> {
        let mut character_ids = Vec::new();
        
        // Ensure directory exists
        if !self.save_directory.exists() {
            return Ok(character_ids);
        }
        
        // Read directory entries
        let mut entries = tokio::fs::read_dir(&self.save_directory).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            // Check if it's a JSON file
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    character_ids.push(stem.to_string());
                }
            }
        }
        
        Ok(character_ids)
    }
    
    /// Delete a character save file
    pub async fn delete_character(&self, character_id: &str) -> Result<(), SaveError> {
        let filename = format!("{}.json", character_id);
        let filepath = self.save_directory.join(&filename);
        
        if !filepath.exists() {
            return Err(SaveError::NotFound(filename));
        }
        
        tokio::fs::remove_file(filepath).await?;
        Ok(())
    }
    
    /// Check if a character save exists
    pub fn character_exists(&self, character_id: &str) -> bool {
        let filename = format!("{}.json", character_id);
        let filepath = self.save_directory.join(filename);
        filepath.exists()
    }
}

// ============================================================================
// SAVE FILE FORMAT
// ============================================================================

// The save file format is JSON serialization of the Character struct
// All UUIDs are preserved exactly as they were when saving
// This ensures:
// - Character state is fully preserved across save/load cycles
// - Effects reference the same definition IDs after loading
// - Affinities and attributes maintain their instance UUIDs
// - Cross-references remain valid after deserialization
//
// Example save file structure:
// {
//   "id": "550e8400-e29b-41d4-a716-446655440000",
//   "name": "Test Character",
//   "current_loop": 1,
//   "current_tick": 1000,
//   "affinities": { ... },
//   "attributes": { ... },
//   "effects": [ ... ]
// }
