// Game instance structures (runtime state)
// This includes: AffinityInstance, AttributeInstance, Effect, Character, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::models::common::*;

// ============================================================================
// EFFECT INSTANCE
// ============================================================================

/// Universal effect that can modify any aspect of any game object instance
/// Effects are stored on the objects they affect and reference their definition
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Effect {
    pub id: EffectId,                   // Effect instance UUID
    pub definition_id: EffectDefinitionId, // References EffectDefinition
    pub source_id: CharacterId,         // What/who caused this effect
    pub applied_at: GameTick,           // When effect was applied (game time)
    
    // Instance-specific data (copied from definition, then tracked independently)
    pub duration: Option<u64>,          // Duration in ticks (None = permanent)
    pub conditions: Vec<String>,        // When effect is active (simple strings for now)
}

impl Effect {
    /// Create new effect instance from a definition
    pub fn new_from_definition(
        definition_id: EffectDefinitionId,
        source_id: CharacterId,
        applied_at: GameTick,
        duration: Option<u64>,
        conditions: Vec<String>,
    ) -> Self {
        Self {
            id: EffectId::new(),        // Generate new UUID for instance
            definition_id,
            source_id,
            applied_at,
            duration,
            conditions,
        }
    }
    
    /// Load effect from save data (preserves UUID)
    pub fn from_save_data(
        id: EffectId,
        definition_id: EffectDefinitionId,
        source_id: CharacterId,
        applied_at: GameTick,
        duration: Option<u64>,
        conditions: Vec<String>,
    ) -> Self {
        Self {
            id,
            definition_id,
            source_id,
            applied_at,
            duration,
            conditions,
        }
    }
    
    /// Check if effect is still active
    pub fn is_active(&self, current_time: GameTick) -> bool {
        match self.duration {
            None => true, // Permanent effect
            Some(duration) => {
                let end_time = self.applied_at.add(duration);
                current_time.get() < end_time.get()
            }
        }
    }
    
    /// Get remaining duration in ticks
    pub fn remaining_duration(&self, current_time: GameTick) -> Option<u64> {
        match self.duration {
            None => None, // Permanent
            Some(duration) => {
                let end_time = self.applied_at.add(duration);
                Some(end_time.elapsed_since(current_time))
            }
        }
    }
}

// ============================================================================
// AFFINITY INSTANCE
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AffinityInstance {
    pub id: CharacterId,                // Instance UUID
    pub definition_id: AffinityId,      // References AffinityDefinition
    pub acquired_at: GameTick,          // When this affinity was acquired
    pub base_mastery_level: u32,        // Base mastery level (without effects)
    pub effects: Vec<Effect>,           // Effects that modify this affinity
}

impl AffinityInstance {
    /// Create new affinity instance from a definition
    pub fn new_from_definition(
        definition_id: AffinityId,
        acquired_at: GameTick,
    ) -> Self {
        Self {
            id: CharacterId::new(),     // Generate new UUID
            definition_id,
            acquired_at,
            base_mastery_level: 0,      // Start at level 0
            effects: Vec::new(),
        }
    }
    
    /// Load affinity from save data (preserves UUID)
    pub fn from_save_data(
        id: CharacterId,
        definition_id: AffinityId,
        acquired_at: GameTick,
        base_mastery_level: u32,
        effects: Vec<Effect>,
    ) -> Self {
        Self {
            id,
            definition_id,
            acquired_at,
            base_mastery_level,
            effects,
        }
    }
    
    /// Calculate effective mastery level (base + effects)
    pub fn effective_mastery_level(&self, current_time: GameTick) -> u32 {
        // Start with base value
        let total = self.base_mastery_level as i64;
        
        // Apply active effects
        for effect in &self.effects {
            if effect.is_active(current_time) {
                // TODO: Apply effect modifications when we implement EffectCollector
                // For now, just use base value
            }
        }
        
        // Ensure non-negative
        total.max(0) as u32
    }
}

// ============================================================================
// ATTRIBUTE INSTANCE
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttributeInstance {
    pub id: CharacterId,                // Instance UUID
    pub definition_id: AttributeId,     // References AttributeDefinition
    pub base_value: u32,                // Base attribute value (without effects)
    pub training_mode: TrainingMode,    // Current training state
    pub effects: Vec<Effect>,           // Effects that modify this attribute
}

impl AttributeInstance {
    /// Create new attribute instance from a definition
    pub fn new_from_definition(
        definition_id: AttributeId,
        base_value: u32,
    ) -> Self {
        Self {
            id: CharacterId::new(),     // Generate new UUID
            definition_id,
            base_value,
            training_mode: TrainingMode::None,
            effects: Vec::new(),
        }
    }
    
    /// Load attribute from save data (preserves UUID)
    pub fn from_save_data(
        id: CharacterId,
        definition_id: AttributeId,
        base_value: u32,
        training_mode: TrainingMode,
        effects: Vec<Effect>,
    ) -> Self {
        Self {
            id,
            definition_id,
            base_value,
            training_mode,
            effects,
        }
    }
    
    /// Calculate effective attribute value (base + effects)
    pub fn effective_value(&self, current_time: GameTick) -> u32 {
        // Start with base value
        let total = self.base_value as i64;
        
        // Apply active effects
        for effect in &self.effects {
            if effect.is_active(current_time) {
                // TODO: Apply effect modifications when we implement EffectCollector
                // For now, just use base value
            }
        }
        
        // Ensure non-negative
        total.max(0) as u32
    }
    
    /// Start training this attribute
    pub fn start_training(&mut self) {
        self.training_mode = TrainingMode::Active;
    }
    
    /// Stop training this attribute
    pub fn stop_training(&mut self) {
        self.training_mode = TrainingMode::None;
    }
}

// ============================================================================
// CHARACTER (TIMELOOPER)
// ============================================================================

/// The player character (Timelooper)
/// Aggregates all character state and owned objects
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Character {
    pub id: CharacterId,                            // Character UUID
    pub name: String,                               // Character name
    pub current_loop: u32,                          // Current loop number
    pub current_tick: GameTick,                     // Current game time
    
    // Core progression
    pub affinities: HashMap<AffinityId, AffinityInstance>, // Acquired affinities
    pub attributes: HashMap<AttributeId, AttributeInstance>, // Character attributes
    
    // Effects on the character directly
    pub effects: Vec<Effect>,                       // Character-level effects
}

impl Character {
    /// Create a new character
    pub fn new(name: String, starting_attributes: HashMap<AttributeId, u32>) -> Self {
        let mut attributes = HashMap::new();
        
        // Create attribute instances from starting values
        for (attr_id, base_value) in starting_attributes {
            let instance = AttributeInstance::new_from_definition(attr_id, base_value);
            attributes.insert(attr_id, instance);
        }
        
        Self {
            id: CharacterId::new(),
            name,
            current_loop: 1,
            current_tick: GameTick::zero(),
            affinities: HashMap::new(),
            attributes,
            effects: Vec::new(),
        }
    }
    
    /// Load character from save data
    pub fn from_save_data(
        id: CharacterId,
        name: String,
        current_loop: u32,
        current_tick: GameTick,
        affinities: HashMap<AffinityId, AffinityInstance>,
        attributes: HashMap<AttributeId, AttributeInstance>,
        effects: Vec<Effect>,
    ) -> Self {
        Self {
            id,
            name,
            current_loop,
            current_tick,
            affinities,
            attributes,
            effects,
        }
    }
    
    /// Acquire a new affinity
    pub fn acquire_affinity(&mut self, definition_id: AffinityId) {
        if !self.affinities.contains_key(&definition_id) {
            let affinity = AffinityInstance::new_from_definition(
                definition_id,
                self.current_tick,
            );
            self.affinities.insert(definition_id, affinity);
        }
    }
    
    /// Get an attribute value
    pub fn get_attribute_value(&self, attr_id: &AttributeId) -> Option<u32> {
        self.attributes.get(attr_id).map(|attr| attr.effective_value(self.current_tick))
    }
    
    /// Advance game time
    pub fn advance_tick(&mut self) {
        self.current_tick = self.current_tick.add(1);
    }
}
