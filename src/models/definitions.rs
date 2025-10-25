// Game definition structures (immutable templates)
// This includes: AffinityDefinition, AttributeDefinition, EffectDefinition, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::models::common::*;

// ============================================================================
// AFFINITY DEFINITION
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AffinityDefinition {
    pub id: AffinityId,                                 // Definition identifier (from external file)
    pub name: String,                                   // e.g., "Hand-to-Hand Combat", "Psionic"
    pub description: String,                            // Short description
    pub lore_text: String,                              // Background story/flavor text
    pub attribute_bonuses: HashMap<AttributeId, u32>,   // Physical/Mental attribute bonuses
    pub mastery_benefits: HashMap<u32, String>,         // Mastery level -> benefit description
    pub icon: Option<String>,                           // Icon filename/path
}

// ============================================================================
// ATTRIBUTE DEFINITION
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttributeDefinition {
    pub id: AttributeId,                // Definition identifier (from external file)
    pub name: String,                   // e.g., "Strength", "Intelligence"
    pub description: String,            // What this attribute represents
    pub category: AttributeCategory,    // Physical, Mental, Social
    pub base_value: u32,                // Default starting value for new characters
    pub min_value: u32,                 // Minimum possible value
    pub max_value: u32,                 // Maximum possible value
    pub training_difficulty: Percentage, // How hard to improve (100% = normal, 150% = harder)
    pub icon: Option<String>,           // Icon filename/path
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttributeCategory {
    Physical,
    Mental,
    Social,
}

// ============================================================================
// EFFECT DEFINITION
// ============================================================================

/// Effect definition - immutable template for effects
/// Contains all the fixed properties that define what an effect does
/// Instance Effects reference this via definition_id and track their own state
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EffectDefinition {
    pub id: EffectDefinitionId,         // Definition identifier (from external file)
    pub name: String,                   // e.g., "Strength Boost", "Training Speed Up"
    pub description: String,            // What this effect does
    
    // Effect mechanics (immutable - defines what this effect is)
    pub property: Property,             // What property this effect modifies
    pub modification: EffectModification, // How it modifies the property
    
    // Default instance values (copied to instances at creation)
    pub default_duration: Option<u64>,  // Default duration in ticks (None = permanent)
    pub default_conditions: Vec<String>, // Default conditions for effect activation
    
    // Effect behavior
    pub stacking: StackingBehavior,     // How multiple instances stack
    pub priority: i32,                  // Application order (higher = applies later)
    
    // Visual/UI
    pub icon: Option<String>,           // Icon filename/path
    pub visual_effect: Option<String>,  // Visual effect name for rendering
}

// Design notes:
// - Property and modification are immutable: changing them creates a different effect
// - Duration and conditions are copied to instances: instances track their own state
// - Stacking behavior is part of effect identity: different stacking = different effect
// - Priority determines order of effect application within stacking groups

// ============================================================================
// SKILL DEFINITION (Future - not in kickoff phase)
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SkillDefinition {
    pub id: SkillId,                    // Definition identifier
    pub name: String,                   // e.g., "Punch", "Fireball"
    pub description: String,            // What this skill does
    pub affinity_requirement: Option<AffinityId>, // Required affinity
    pub attribute_requirements: HashMap<AttributeId, u32>, // Required attributes
    pub cooldown_ticks: u64,           // Cooldown in game ticks
    pub icon: Option<String>,          // Icon filename
}

// ============================================================================
// MASTERY DEFINITION (Future - not in kickoff phase)
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MasteryDefinition {
    pub id: MasteryId,                  // Definition identifier
    pub name: String,                   // e.g., "Combat Mastery"
    pub description: String,            // What mastery provides
    pub affinity_id: AffinityId,        // Which affinity this mastery belongs to
    pub level_benefits: HashMap<u32, Vec<EffectDefinitionId>>, // Level -> effects gained
    pub icon: Option<String>,           // Icon filename
}
