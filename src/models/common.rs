// Common types and utilities shared across the models module
// This includes: Percentage, timestamps, common enums, etc.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// PERCENTAGE TYPE - Deterministic ordinal type for game mechanics
// ============================================================================

/// Deterministic percentage type using whole number percentages
/// Minimum unit: 1% (no decimal precision needed for game mechanics)
/// Range: 0 to u16::MAX (0% to 65535% - no ceiling for multiplier stacking)
/// Examples: 1 = 1%, 50 = 50%, 100 = 100%, 150 = 150% (1.5x multiplier)
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Percentage(u16); // Whole number percentages, no ceiling

impl Percentage {
    /// Create from raw percentage value (e.g., 50 = 50%, 150 = 150%)
    pub const fn new(percent: u16) -> Self {
        Self(percent)
    }
    
    /// Zero percent (0%)
    pub const fn zero() -> Self {
        Self(0)
    }
    
    /// One hundred percent (100%)
    pub const fn one_hundred() -> Self {
        Self(100)
    }
    
    /// Get raw percentage value
    pub const fn get(&self) -> u16 {
        self.0
    }
    
    /// Apply this percentage to a value (deterministic integer math)
    /// Example: 50% of 200 = (200 * 50) / 100 = 100
    /// Example: 150% of 200 = (200 * 150) / 100 = 300
    pub fn apply_to(&self, value: u64) -> u64 {
        (value * self.0 as u64) / 100
    }
    
    /// Add two percentages (no ceiling - can exceed 100%)
    pub fn add(&self, other: Percentage) -> Self {
        Self(self.0.saturating_add(other.0))
    }
    
    /// Subtract two percentages (floored at 0%)
    pub fn subtract(&self, other: Percentage) -> Self {
        Self(self.0.saturating_sub(other.0))
    }
    
    /// Multiply percentage by scalar (for stacking multipliers)
    /// Example: 150% * 2 = 300%
    pub fn multiply_scalar(&self, scalar: u16) -> Self {
        Self(self.0.saturating_mul(scalar))
    }
    
    /// Multiply two percentages together (for compounding effects)
    /// Example: 150% * 120% = (150 * 120) / 100 = 180%
    pub fn multiply(&self, other: Percentage) -> Self {
        let result = (self.0 as u32 * other.0 as u32) / 100;
        Self(result.min(u16::MAX as u32) as u16)
    }
}

// Implement From/Into traits for implicit conversions
impl From<u16> for Percentage {
    fn from(percent: u16) -> Self {
        Self(percent)
    }
}

impl From<Percentage> for u16 {
    fn from(percentage: Percentage) -> Self {
        percentage.0
    }
}

impl From<u8> for Percentage {
    fn from(percent: u8) -> Self {
        Self(percent as u16)
    }
}

impl From<u64> for Percentage {
    fn from(percent: u64) -> Self {
        Self(percent.min(u16::MAX as u64) as u16)
    }
}

// Common percentage constants
impl Percentage {
    pub const ZERO: Percentage = Percentage(0);                  // 0%
    pub const TEN: Percentage = Percentage(10);                  // 10%
    pub const TWENTY_FIVE: Percentage = Percentage(25);          // 25%
    pub const FIFTY: Percentage = Percentage(50);                // 50%
    pub const SEVENTY_FIVE: Percentage = Percentage(75);         // 75%
    pub const ONE_HUNDRED: Percentage = Percentage(100);         // 100%
    pub const ONE_TWENTY_FIVE: Percentage = Percentage(125);     // 125%
    pub const ONE_FIFTY: Percentage = Percentage(150);           // 150% (1.5x multiplier)
    pub const TWO_HUNDRED: Percentage = Percentage(200);         // 200% (2x multiplier)
}

// ============================================================================
// GAME TIME TYPE - Deterministic tick-based time
// ============================================================================

/// Game time measured in ticks (deterministic, serializable)
/// Used for all in-game timing to avoid DateTime issues
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GameTick(u64);

impl GameTick {
    pub const fn new(tick: u64) -> Self {
        Self(tick)
    }
    
    pub const fn zero() -> Self {
        Self(0)
    }
    
    pub const fn get(&self) -> u64 {
        self.0
    }
    
    pub fn add(&self, ticks: u64) -> Self {
        Self(self.0.saturating_add(ticks))
    }
    
    pub fn subtract(&self, ticks: u64) -> Self {
        Self(self.0.saturating_sub(ticks))
    }
    
    pub fn elapsed_since(&self, earlier: GameTick) -> u64 {
        self.0.saturating_sub(earlier.0)
    }
}

impl From<u64> for GameTick {
    fn from(tick: u64) -> Self {
        Self(tick)
    }
}

impl From<GameTick> for u64 {
    fn from(tick: GameTick) -> Self {
        tick.0
    }
}

// ============================================================================
// ID TYPES - Type-safe UUID wrappers
// ============================================================================

/// Base trait for all ID types
pub trait GameId {
    fn new() -> Self;
    fn from_uuid(uuid: Uuid) -> Self;
    fn as_uuid(&self) -> &Uuid;
}

// Macro to generate type-safe ID wrappers
macro_rules! define_id {
    ($name:ident) => {
        #[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name(Uuid);
        
        impl $name {
            /// Create a new random ID (only for instances)
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            
            /// Create from existing UUID (for loading from JSON/save files)
            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }
            
            /// Get the underlying UUID
            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }
        }
        
        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
        
        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }
        
        impl From<$name> for Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
}

// Definition IDs (loaded from external JSON)
define_id!(AffinityId);
define_id!(AttributeId);
define_id!(EffectDefinitionId);
define_id!(SkillId);
define_id!(MasteryId);

// Instance IDs (created at runtime or loaded from saves)
define_id!(CharacterId);
define_id!(EffectId);

// ============================================================================
// COMMON ENUMS
// ============================================================================

/// Property that effects can modify
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Property {
    /// Attribute values
    Attribute(AttributeId),
    /// Training speed for specific attribute
    AttributeTrainingSpeed(AttributeId),
    /// Experience gain multiplier
    ExperienceGain,
    /// Health/stamina/mana regeneration
    HealthRegen,
    StaminaRegen,
    ManaRegen,
}

/// How an effect modifies a property
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum EffectModification {
    /// Add/subtract flat value (e.g., +10 strength)
    Additive(i32),
    /// Multiply by percentage (e.g., 150% = 1.5x multiplier)
    Multiplicative(Percentage),
    /// Percentage of base value (e.g., 50% of base strength)
    PercentageOfBase(Percentage),
}

/// Effect stacking behavior
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StackingBehavior {
    /// Only one instance can be active (newer replaces older)
    Replace,
    /// Multiple instances stack additively (effects sum)
    StackAdditive,
    /// Multiple instances stack multiplicatively (effects multiply)
    StackMultiplicative,
    /// Each instance tracked separately with independent duration
    Independent,
}

/// Training mode for attributes
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TrainingMode {
    /// Not actively training
    None,
    /// Training this attribute
    Active,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Check if a UUID string is valid
pub fn is_valid_uuid(s: &str) -> bool {
    Uuid::parse_str(s).is_ok()
}
