// I/O operations - Load and save attributes

use crate::models::definitions::AttributeDefinition;
use std::path::{Path, PathBuf};
use std::fs;

/// Load attributes from JSON file
pub fn load_attributes() -> Result<Vec<AttributeDefinition>, String> {
    let path = get_attributes_path();
    
    if !path.exists() {
        return Err(format!("Attributes file not found: {}", path.display()));
    }
    
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read attributes file: {}", e))?;
    
    let attributes: Vec<AttributeDefinition> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse attributes JSON: {}", e))?;
    
    Ok(attributes)
}

/// Save attributes to JSON file
pub fn save_attributes(attributes: &[AttributeDefinition]) -> Result<(), String> {
    let path = get_attributes_path();
    
    // Create backup first
    backup_attributes(&path)?;
    
    // Serialize with pretty printing
    let json = serde_json::to_string_pretty(attributes)
        .map_err(|e| format!("Failed to serialize attributes: {}", e))?;
    
    // Write to temporary file first (atomic operation)
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, json)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;
    
    // Rename temp to actual (atomic on most systems)
    fs::rename(&temp_path, &path)
        .map_err(|e| format!("Failed to save attributes: {}", e))?;
    
    Ok(())
}

/// Create backup of current attributes file
fn backup_attributes(path: &Path) -> Result<(), String> {
    if path.exists() {
        let backup_path = path.with_extension("json.backup");
        fs::copy(path, &backup_path)
            .map_err(|e| format!("Failed to create backup: {}", e))?;
    }
    Ok(())
}

/// Get path to attributes.json
fn get_attributes_path() -> PathBuf {
    PathBuf::from("game_data/core/attributes.json")
}
