// Core simulation loop and time advancement
// This includes: GameState, tick processing, action execution

use crate::models::common::*;
use crate::models::instances::*;
use crate::storage::GameDefinitionsLoader;

// ============================================================================
// GAME STATE
// ============================================================================

/// Main game state container
/// Manages character state and game progression
pub struct GameState {
    pub character: Character,
    pub definitions: GameDefinitionsLoader,
    pub tick_rate: u64,  // Milliseconds per tick (for real-time servers)
}

impl GameState {
    /// Create a new game state with a character and definitions
    pub fn new(character: Character, definitions: GameDefinitionsLoader) -> Self {
        Self {
            character,
            definitions,
            tick_rate: 1000, // Default: 1 second per tick
        }
    }
    
    /// Advance the game by one tick
    pub fn advance_tick(&mut self) -> TickResult {
        let current_tick = self.character.current_tick;
        
        // Advance time
        self.character.advance_tick();
        
        // Process training for active attributes
        let mut trained_attributes = Vec::new();
        for (attr_id, attribute) in &mut self.character.attributes {
            if matches!(attribute.training_mode, TrainingMode::Active) {
                // Get attribute definition for training difficulty
                if let Some(def) = self.definitions.get_attribute_definition(attr_id) {
                    let progress = crate::game_engine::progression::calculate_attribute_training_progress(
                        attribute,
                        def.training_difficulty,
                        current_tick,
                    );
                    
                    // Apply progress
                    if crate::game_engine::progression::apply_attribute_training(
                        attribute,
                        progress,
                        def.max_value,
                        crate::game_engine::progression::DEFAULT_ATTRIBUTE_PROGRESS_THRESHOLD,
                    ) {
                        trained_attributes.push(*attr_id);
                    }
                }
            }
        }
        
        // Process affinity mastery progression
        let mut mastered_affinities = Vec::new();
        for (aff_id, affinity) in &mut self.character.affinities {
            let progress = crate::game_engine::progression::calculate_affinity_mastery_progress(
                affinity,
                current_tick,
            );
            
            if crate::game_engine::progression::apply_affinity_mastery(
                affinity,
                progress,
                crate::game_engine::progression::DEFAULT_MAX_MASTERY_LEVEL,
                crate::game_engine::progression::DEFAULT_MASTERY_PROGRESS_THRESHOLD,
            ) {
                mastered_affinities.push(*aff_id);
            }
        }
        
        // Clean up expired effects
        self.clean_expired_effects();
        
        TickResult {
            tick: self.character.current_tick,
            trained_attributes,
            mastered_affinities,
            events: Vec::new(),
        }
    }
    
    /// Remove expired effects from all objects
    fn clean_expired_effects(&mut self) {
        let current_tick = self.character.current_tick;
        
        // Clean character effects
        self.character.effects.retain(|e| e.is_active(current_tick));
        
        // Clean attribute effects
        for attribute in self.character.attributes.values_mut() {
            attribute.effects.retain(|e| e.is_active(current_tick));
        }
        
        // Clean affinity effects
        for affinity in self.character.affinities.values_mut() {
            affinity.effects.retain(|e| e.is_active(current_tick));
        }
    }
    
    /// Get current game tick
    pub fn current_tick(&self) -> GameTick {
        self.character.current_tick
    }
}

// ============================================================================
// TICK RESULT
// ============================================================================

/// Result of processing a single game tick
#[derive(Debug, Clone)]
pub struct TickResult {
    pub tick: GameTick,
    pub trained_attributes: Vec<AttributeId>,
    pub mastered_affinities: Vec<AffinityId>,
    pub events: Vec<GameEvent>,
}

// ============================================================================
// GAME EVENTS
// ============================================================================

/// Events that occur during game simulation
#[derive(Debug, Clone)]
pub enum GameEvent {
    /// Attribute increased
    AttributeIncreased {
        attribute_id: AttributeId,
        old_value: u32,
        new_value: u32,
    },
    /// Affinity mastery increased
    MasteryIncreased {
        affinity_id: AffinityId,
        old_level: u32,
        new_level: u32,
    },
    /// Effect expired
    EffectExpired {
        effect_id: EffectId,
    },
    /// Effect applied
    EffectApplied {
        effect_id: EffectId,
        target: EffectTarget,
    },
}

#[derive(Debug, Clone)]
pub enum EffectTarget {
    Character,
    Attribute(AttributeId),
    Affinity(AffinityId),
}

// ============================================================================
// ACTION PROCESSING
// ============================================================================

/// Actions that can be performed in the game
#[derive(Debug, Clone)]
pub enum GameAction {
    /// Start training an attribute
    StartTraining { attribute_id: AttributeId },
    /// Stop training an attribute
    StopTraining { attribute_id: AttributeId },
    /// Acquire a new affinity
    AcquireAffinity { affinity_id: AffinityId },
    /// Apply an effect to a target
    ApplyEffect {
        effect_definition_id: EffectDefinitionId,
        target: EffectTarget,
    },
}

impl GameState {
    /// Execute a game action
    pub fn execute_action(&mut self, action: GameAction) -> Result<(), ActionError> {
        match action {
            GameAction::StartTraining { attribute_id } => {
                if let Some(attribute) = self.character.attributes.get_mut(&attribute_id) {
                    attribute.start_training();
                    Ok(())
                } else {
                    Err(ActionError::Attribute(attribute_id))
                }
            }
            GameAction::StopTraining { attribute_id } => {
                if let Some(attribute) = self.character.attributes.get_mut(&attribute_id) {
                    attribute.stop_training();
                    Ok(())
                } else {
                    Err(ActionError::Attribute(attribute_id))
                }
            }
            GameAction::AcquireAffinity { affinity_id } => {
                // Check if affinity definition exists
                if !self.definitions.affinities.contains_key(&affinity_id) {
                    return Err(ActionError::Affinity(affinity_id));
                }
                
                // Acquire the affinity
                self.character.acquire_affinity(affinity_id);
                Ok(())
            }
            GameAction::ApplyEffect { effect_definition_id, target } => {
                // Create effect instance
                let effect = self.definitions.create_effect_instance(
                    effect_definition_id,
                    self.character.id,
                    self.character.current_tick,
                ).map_err(|_| ActionError::EffectDefinition(effect_definition_id))?;
                
                // Apply to target
                match target {
                    EffectTarget::Character => {
                        self.character.effects.push(effect);
                    }
                    EffectTarget::Attribute(attr_id) => {
                        if let Some(attribute) = self.character.attributes.get_mut(&attr_id) {
                            attribute.effects.push(effect);
                        } else {
                            return Err(ActionError::Attribute(attr_id));
                        }
                    }
                    EffectTarget::Affinity(aff_id) => {
                        if let Some(affinity) = self.character.affinities.get_mut(&aff_id) {
                            affinity.effects.push(effect);
                        } else {
                            return Err(ActionError::Affinity(aff_id));
                        }
                    }
                }
                
                Ok(())
            }
        }
    }
}

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ActionError {
    #[error("Attribute not found: {0:?}")]
    Attribute(AttributeId),
    
    #[error("Affinity not found: {0:?}")]
    Affinity(AffinityId),
    
    #[error("Effect definition not found: {0:?}")]
    EffectDefinition(EffectDefinitionId),
}
