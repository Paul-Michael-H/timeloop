// Effect system implementation
// This includes: EffectCollector, effect application, stacking logic

use crate::models::common::*;
use crate::models::instances::Effect;

// ============================================================================
// EFFECT COLLECTOR
// ============================================================================

/// Collects and applies effects from multiple sources
/// Uses deterministic integer math for all calculations
pub struct EffectCollector {
    effects: Vec<Effect>,
}

impl EffectCollector {
    /// Create a new empty effect collector
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }
    
    /// Add effects from a collection if they're active
    pub fn add_effects(&mut self, effects: &[Effect], current_time: GameTick) {
        for effect in effects {
            if effect.is_active(current_time) {
                self.effects.push(effect.clone());
            }
        }
    }
    
    /// Add a single effect if it's active
    pub fn add_effect(&mut self, effect: &Effect, current_time: GameTick) {
        if effect.is_active(current_time) {
            self.effects.push(effect.clone());
        }
    }
    
    /// Calculate the final effective value for a property
    /// This is where all effect magic happens - deterministic and generic!
    pub fn calculate_effective_value(&self, property: &Property, base_value: u64) -> u64 {
        // Filter effects for this property
        let applicable_effects: Vec<_> = self.effects.iter()
            .filter(|e| self.matches_property(e, property))
            .collect();
        
        if applicable_effects.is_empty() {
            return base_value;
        }
        
        // Apply effects with stacking rules
        self.apply_effects_with_stacking(&applicable_effects, base_value)
    }
    
    /// Check if an effect's property matches the requested property
    fn matches_property(&self, _effect: &Effect, _property: &Property) -> bool {
        // TODO: In real implementation, would look up effect definition and compare
        // For now, return false since we need definition lookup
        false
    }
    
    /// Apply effects respecting stacking behavior
    fn apply_effects_with_stacking(&self, _effects: &[&Effect], base_value: u64) -> u64 {
        // TODO: Group effects by stacking behavior and apply
        // For now, just return base value since we need definition lookup
        base_value
    }
    
    /// Get count of effects currently collected
    pub fn effect_count(&self) -> usize {
        self.effects.len()
    }
}

impl Default for EffectCollector {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// EFFECT APPLICATION HELPERS
// ============================================================================

/// Helper to apply a single modification to a value
pub fn apply_modification(modification: &EffectModification, base_value: u64) -> u64 {
    match modification {
        EffectModification::Additive(amount) => {
            base_value.saturating_add_signed(*amount as i64)
        }
        EffectModification::Multiplicative(percentage) => {
            percentage.apply_to(base_value)
        }
        EffectModification::PercentageOfBase(percentage) => {
            let bonus = percentage.apply_to(base_value);
            base_value.saturating_add(bonus)
        }
    }
}

// ============================================================================
// EFFECT AGGREGATOR TRAIT
// ============================================================================

/// Trait for any object that can contribute effects to calculations
/// Objects implement this to expose their effects to the collection system
pub trait EffectAggregator {
    /// Collect effects from this object into the collector
    fn collect_effects(&self, collector: &mut EffectCollector, current_time: GameTick);
}
