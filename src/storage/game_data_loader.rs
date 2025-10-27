// Game data loading from JSON files
// This includes: GameDefinitionsLoader and external data structure parsing

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use crate::models::common::*;
use crate::models::definitions::*;
use crate::models::instances::*;

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Missing required file: {0}")]
    MissingFile(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid reference: {entity_type} {entity_id} references missing {ref_type} {ref_id}")]
    InvalidReference {
        entity_type: String,
        entity_id: String,
        ref_type: String,
        ref_id: String,
    },
    
    #[error("Duplicate ID: {entity_type} {id}")]
    DuplicateId {
        entity_type: String,
        id: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum CreationError {
    #[error("Definition not found: {0}")]
    DefinitionNotFound(String),
}

// ============================================================================
// GAME DEFINITIONS LOADER
// ============================================================================

/// Loads and manages all game definitions from external JSON files
/// Definitions are immutable templates loaded once at startup
#[derive(Debug, Clone)]
pub struct GameDefinitionsLoader {
    pub affinities: HashMap<AffinityId, AffinityDefinition>,
    pub attributes: HashMap<AttributeId, AttributeDefinition>,
    pub effects: HashMap<EffectDefinitionId, EffectDefinition>,
    // Future: skills, masteries, items, etc.
}

impl GameDefinitionsLoader {
    /// Create a new empty loader (for testing)
    pub fn new() -> Self {
        Self {
            affinities: HashMap::new(),
            attributes: HashMap::new(),
            effects: HashMap::new(),
        }
    }
    
    /// Load all definitions from JSON files in the specified directory
    pub async fn load_all_definitions(data_path: &Path) -> Result<Self, LoadError> {
        // Load attributes using the editor's I/O module for consistency
        let attributes = crate::editor::io::load_attributes()
            .map_err(|e| LoadError::MissingFile(format!("attributes.json: {}", e)))?;
        
        // Load other definition types from their JSON files
        let affinities_path = data_path.join("affinities.json");
        let effects_path = data_path.join("effects.json");
        
        // Try to load affinities (skip if fails - not in Phase 1)
        let affinities = Self::load_json_file::<Vec<AffinityDefinition>>(&affinities_path)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to load affinities: {:?} - using empty list", e);
                Vec::new()
            });
        
        // Try to load effects (skip if fails - not in Phase 1)
        let effects = Self::load_json_file::<Vec<EffectDefinition>>(&effects_path)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to load effects: {:?} - using empty list", e);
                Vec::new()
            });
        
        // Convert to HashMaps
        let attributes_map: HashMap<AttributeId, AttributeDefinition> = 
            attributes.into_iter().map(|def| (def.id, def)).collect();
        let affinities_map: HashMap<AffinityId, AffinityDefinition> = 
            affinities.into_iter().map(|def| (def.id, def)).collect();
        let effects_map: HashMap<EffectDefinitionId, EffectDefinition> = 
            effects.into_iter().map(|def| (def.id, def)).collect();
        
        Ok(Self {
            attributes: attributes_map,
            affinities: affinities_map,
            effects: effects_map,
        })
    }
    
    /// Helper to load and parse a JSON file
    async fn load_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, LoadError> {
        let content = tokio::fs::read_to_string(path).await?;
        let data = serde_json::from_str(&content)?;
        Ok(data)
    }
    
    /// Validate that all cross-references between definitions are valid
    pub fn validate_definition_references(&self) -> Result<(), ValidationError> {
        // Validate affinity -> attribute references
        for (affinity_id, affinity) in &self.affinities {
            for attribute_id in affinity.attribute_bonuses.keys() {
                if !self.attributes.contains_key(attribute_id) {
                    return Err(ValidationError::InvalidReference {
                        entity_type: "AffinityDefinition".to_string(),
                        entity_id: format!("{:?}", affinity_id),
                        ref_type: "AttributeDefinition".to_string(),
                        ref_id: format!("{:?}", attribute_id),
                    });
                }
            }
        }
        
        // Validate effect -> attribute references (in Property enum)
        for (effect_id, effect) in &self.effects {
            if let Property::Attribute(attr_id) = effect.property {
                if !self.attributes.contains_key(&attr_id) {
                    return Err(ValidationError::InvalidReference {
                        entity_type: "EffectDefinition".to_string(),
                        entity_id: format!("{:?}", effect_id),
                        ref_type: "AttributeDefinition".to_string(),
                        ref_id: format!("{:?}", attr_id),
                    });
                }
            }
            
            if let Property::AttributeTrainingSpeed(attr_id) = effect.property {
                if !self.attributes.contains_key(&attr_id) {
                    return Err(ValidationError::InvalidReference {
                        entity_type: "EffectDefinition".to_string(),
                        entity_id: format!("{:?}", effect_id),
                        ref_type: "AttributeDefinition".to_string(),
                        ref_id: format!("{:?}", attr_id),
                    });
                }
            }
        }
        
        Ok(())
    }
    
    // ========================================================================
    // DEFINITION ACCESS METHODS
    // ========================================================================
    
    pub fn get_affinity_definition(&self, id: &AffinityId) -> Option<&AffinityDefinition> {
        self.affinities.get(id)
    }
    
    pub fn get_attribute_definition(&self, id: &AttributeId) -> Option<&AttributeDefinition> {
        self.attributes.get(id)
    }
    
    pub fn get_effect_definition(&self, id: &EffectDefinitionId) -> Option<&EffectDefinition> {
        self.effects.get(id)
    }
    
    pub fn list_all_affinities(&self) -> Vec<&AffinityDefinition> {
        self.affinities.values().collect()
    }
    
    pub fn list_all_attributes(&self) -> Vec<&AttributeDefinition> {
        self.attributes.values().collect()
    }
    
    pub fn list_all_effects(&self) -> Vec<&EffectDefinition> {
        self.effects.values().collect()
    }
    
    // ========================================================================
    // INSTANCE FACTORY METHODS (Generate NEW UUIDs for fresh instances)
    // ========================================================================
    
    /// Create a new affinity instance from a definition (generates fresh UUID)
    pub fn create_affinity_instance(
        &self,
        definition_id: AffinityId,
        acquired_at: GameTick,
    ) -> Result<AffinityInstance, CreationError> {
        // Verify definition exists
        if !self.affinities.contains_key(&definition_id) {
            return Err(CreationError::DefinitionNotFound(format!("{:?}", definition_id)));
        }
        
        Ok(AffinityInstance::new_from_definition(definition_id, acquired_at))
    }
    
    /// Create a new attribute instance from a definition (generates fresh UUID)
    pub fn create_attribute_instance(
        &self,
        definition_id: AttributeId,
    ) -> Result<AttributeInstance, CreationError> {
        let definition = self.get_attribute_definition(&definition_id)
            .ok_or_else(|| CreationError::DefinitionNotFound(format!("{:?}", definition_id)))?;
        
        // Use the default value from the definition
        Ok(AttributeInstance::new_from_definition(definition_id, definition.base_value))
    }
    
    /// Create a new effect instance from a definition (generates fresh UUID)
    pub fn create_effect_instance(
        &self,
        definition_id: EffectDefinitionId,
        source_id: CharacterId,
        applied_at: GameTick,
    ) -> Result<Effect, CreationError> {
        let definition = self.get_effect_definition(&definition_id)
            .ok_or_else(|| CreationError::DefinitionNotFound(format!("{:?}", definition_id)))?;
        
        // Copy default duration and conditions from definition
        Ok(Effect::new_from_definition(
            definition_id,
            source_id,
            applied_at,
            definition.default_duration,
            definition.default_conditions.clone(),
        ))
    }
}

impl Default for GameDefinitionsLoader {
    fn default() -> Self {
        Self::new()
    }
}
