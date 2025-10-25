// Progression systems: attributes, masteries, training
// This includes: experience calculations, level-up logic, training mechanics

use crate::models::common::*;
use crate::models::instances::*;

// ============================================================================
// ATTRIBUTE PROGRESSION
// ============================================================================

/// Calculate attribute training progress for a single tick
/// Returns the amount of progress made (deterministic)
pub fn calculate_attribute_training_progress(
    attribute: &AttributeInstance,
    training_difficulty: Percentage,
    current_tick: GameTick,
) -> u64 {
    // Base progress per tick (can be tuned for game balance)
    let base_progress = 1u64;
    
    // Apply training difficulty from definition
    // Harder = less progress per tick
    // 100% = normal, 150% = slower, 50% = faster
    let difficulty_modifier = if training_difficulty.get() > 0 {
        // Invert difficulty: higher difficulty = lower progress
        // progress = base * (100 / difficulty)
        (base_progress * 100) / training_difficulty.get() as u64
    } else {
        base_progress
    };
    
    // Apply training speed effects from the attribute instance
    // This would use EffectCollector in full implementation
    let speed_multiplier = attribute_training_speed_multiplier(attribute, current_tick);
    
    // Final progress = difficulty_modified * speed_multiplier
    speed_multiplier.apply_to(difficulty_modifier)
}

/// Get training speed multiplier from attribute effects
fn attribute_training_speed_multiplier(_attribute: &AttributeInstance, _current_tick: GameTick) -> Percentage {
    // TODO: Use EffectCollector to aggregate training speed effects
    // For now, return 100% (no modifiers)
    Percentage::ONE_HUNDRED
}

/// Check if attribute can be trained
pub fn can_train_attribute(attribute: &AttributeInstance, max_value: u32) -> bool {
    attribute.base_value < max_value
}

/// Apply training progress to attribute
/// Returns true if attribute increased in value
pub fn apply_attribute_training(
    attribute: &mut AttributeInstance,
    progress: u64,
    max_value: u32,
    progress_threshold: u64,
) -> bool {
    // Simple progression: accumulate progress until threshold
    // In full implementation, would track progress separately
    if progress >= progress_threshold && attribute.base_value < max_value {
        attribute.base_value += 1;
        true
    } else {
        false
    }
}

// ============================================================================
// AFFINITY PROGRESSION
// ============================================================================

/// Calculate affinity mastery progress for a single tick
pub fn calculate_affinity_mastery_progress(
    affinity: &AffinityInstance,
    current_tick: GameTick,
) -> u64 {
    // Base progress per tick
    let base_progress = 1u64;
    
    // Apply mastery speed effects
    let speed_multiplier = affinity_mastery_speed_multiplier(affinity, current_tick);
    
    speed_multiplier.apply_to(base_progress)
}

/// Get mastery speed multiplier from affinity effects
fn affinity_mastery_speed_multiplier(_affinity: &AffinityInstance, _current_tick: GameTick) -> Percentage {
    // TODO: Use EffectCollector to aggregate mastery speed effects
    // For now, return 100% (no modifiers)
    Percentage::ONE_HUNDRED
}

/// Apply mastery progress to affinity
/// Returns true if mastery level increased
pub fn apply_affinity_mastery(
    affinity: &mut AffinityInstance,
    progress: u64,
    max_mastery: u32,
    progress_threshold: u64,
) -> bool {
    if progress >= progress_threshold && affinity.base_mastery_level < max_mastery {
        affinity.base_mastery_level += 1;
        true
    } else {
        false
    }
}

// ============================================================================
// PROGRESSION CONSTANTS
// ============================================================================

/// Default progress needed to increase attribute by 1
pub const DEFAULT_ATTRIBUTE_PROGRESS_THRESHOLD: u64 = 100;

/// Default progress needed to increase mastery by 1
pub const DEFAULT_MASTERY_PROGRESS_THRESHOLD: u64 = 100;

/// Default maximum attribute value
pub const DEFAULT_MAX_ATTRIBUTE_VALUE: u32 = 100;

/// Default maximum mastery level
pub const DEFAULT_MAX_MASTERY_LEVEL: u32 = 100;
