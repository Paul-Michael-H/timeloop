# Timeloop Game - Kickoff Development Plan

## 📋 **Project Overview**

This plan outlines the step-by-step development approach for the Timeloop game, focusing on delivering a minimal viable foundation with a Timelooper creation system and persistent state management. Each step must achieve 100% test coverage, zero warnings, zero clippy remarks, and a fully working solution.

## 🎯 **Primary Goals for Kickoff Phase**

### **Mandatory Requirements** ⭐
1. **Core Timelooper Data Structure** - Complete entity model with all required game attributes
2. **Main Menu System** - New Game, Load Game, Settings navigation
3. **Timelooper Creation Flow** - Interactive character creation with validation
4. **Persistent State Management** - JSON-based save/load system with UUID identification
5. **Quality Standards** - 100% test coverage, zero warnings, zero clippy issues

### **Nice-to-Have Features** 🌟
1. **Enhanced UI Styling** - Visual polish beyond basic functionality
2. **Configuration System** - Game settings persistence
3. **Advanced Validation** - Complex business rules for character creation
4. **Error Recovery** - Graceful handling of corrupted save files
5. **Performance Optimization** - Async file operations, caching

## 📐 **Technical Architecture Foundation**

### **Data-Driven Architecture Philosophy**
The game engine is **completely data-driven** - all game content (skills, abilities, events, cards, affinities, guilds, houses) is defined in external JSON files and loaded at startup. The engine provides evaluation and simulation capabilities but contains **no hardcoded game content**.

### **Definition vs Instance Architecture**
**CRITICAL SEPARATION**: The architecture maintains strict separation between:

- **Definitions** (Templates/Blueprints): Immutable templates loaded from external files that define what something *can be*
- **Instances** (Actual Objects): Mutable objects with state, history, and individual modifications based on definitions

**Example**:
- **SkillDefinition**: Defines "Swordsmanship" skill with max level 100, learning requirements, etc.
- **SkillInstance**: Timelooper's actual "Swordsmanship" at level 47, with training history, bonuses, etc.
- **Multiple Instances**: Timelooper can have multiple sword items, each based on same ItemDefinition but with different durability, enchantments, history

## 🎯 **CRITICAL: Generic Effect System**

### **Universal Effect Architecture** ⚠️ **MANDATORY**

**Design Philosophy**: One unified effect system that can apply any type of modification to any instantiated game object, replacing all specialized bonus/modifier systems. Effects aggregate automatically from owned objects to their owner through a generic calculation system.

```rust
/// Universal effect that can modify any aspect of any game object instance
/// CONTEXT-AWARE: Effects apply based on WHERE they are stored and aggregate upward:
/// - AttributeInstance.effects -> affect that attribute's calculations
/// - AffinityInstance.effects -> affect that affinity's calculations
/// - ItemInstance.effects (future) -> affect item properties and bubble to owner
/// - Timelooper receives aggregated effects from ALL owned objects automatically
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Effect {
    pub id: Uuid,                       // ⚠️ CRITICAL: Effect instance UUID (explicit management)
    pub definition_id: Uuid,            // ✅ References EffectDefinition for name, description, visuals, property, modification
    pub source_id: Uuid,                // What caused this effect (item, skill, event, etc.)
    pub applied_at: GameTimeStep,       // ⏰ When effect was applied
    
    // Instance-specific data (copied from definition at creation, then tracked independently):
    pub duration: EffectDuration,       // When this effect expires (instance tracks its own lifetime)
    pub conditions: Vec<EffectCondition>, // When effect is active (copied from definition + any additional conditions)
}

impl Effect {
    /// Get property to modify from definition
    pub fn get_property(&self, loader: &GameDefinitionsLoader) -> EffectProperty {
        loader.get_effect_definition(&self.definition_id)
            .map(|def| def.property.clone())
            .unwrap_or(EffectProperty::PrimaryValue) // Fallback shouldn't happen with valid data
    }
    
    /// Get modification from definition
    pub fn get_modification(&self, loader: &GameDefinitionsLoader) -> EffectModification {
        loader.get_effect_definition(&self.definition_id)
            .map(|def| def.modification.clone())
            .unwrap_or(EffectModification::Additive(0))
    }
    
    /// Get priority from definition
    pub fn get_priority(&self, loader: &GameDefinitionsLoader) -> i32 {
        loader.get_effect_definition(&self.definition_id)
            .map(|def| def.priority)
            .unwrap_or(0)
    }
    
    /// Get stacking behavior from definition
    pub fn get_stacking(&self, loader: &GameDefinitionsLoader) -> EffectStacking {
        loader.get_effect_definition(&self.definition_id)
            .map(|def| def.stacking.clone())
            .unwrap_or(EffectStacking::Stack)
    }
    
    /// Add additional condition to this effect instance
    pub fn add_condition(&mut self, condition: EffectCondition) {
        self.conditions.push(condition);
    }
    
    /// Check if effect is still active based on duration
    pub fn is_active(&self, current_time: &GameTimeStep) -> bool {
        // Check duration
        let duration_active = match &self.duration {
            EffectDuration::Permanent => true,
            EffectDuration::UntilGameTime(end_time) => current_time <= end_time,
            EffectDuration::Duration(steps) => {
                let end_time = self.applied_at.add_steps(*steps);
                current_time <= &end_time
            },
            EffectDuration::UntilCondition(_) => true, // TODO: implement condition checking
            EffectDuration::Custom { .. } => true, // TODO: implement custom logic
        };
        
        if !duration_active {
            return false;
        }
        
        // Check all conditions (all must be true)
        self.conditions.iter().all(|condition| {
            match condition {
                EffectCondition::Always => true,
                EffectCondition::GameTimeRange { start, end } => {
                    current_time >= start && current_time <= end
                },
                // TODO: implement other condition types when needed
                _ => true,
            }
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum EffectProperty {
    /// Primary value (context-dependent meaning):
    /// - On Timelooper: money amount
    /// - On AttributeInstance: attribute value
    /// - On AffinityInstance: mastery level
    /// - On ItemInstance: durability/power/etc.
    PrimaryValue,
    /// Training/improvement speed modifier:
    /// - On AttributeInstance: training speed multiplier
    /// - On AffinityInstance: mastery gain multiplier
    TrainingSpeed,
    /// Time-related modifications:
    /// - On Timelooper: time remaining in loop
    TimeModifier,
    /// Loop-related modifications:
    /// - On Timelooper: loop count adjustments
    LoopModifier,
    /// Custom property (extensible for future use)
    Custom { property_name: String },
}

/// Trait for objects that can aggregate effects from their owned objects
/// This is the core pattern that makes effects work without hardcoding property access
pub trait EffectAggregator {
    /// Get all effects from this object and its owned objects
    fn collect_effects(&self, aggregator: &mut EffectCollector, current_time: &GameTimeStep);
}

/// Helper struct that collects and applies effects
#[derive(Default)]
pub struct EffectCollector {
    /// Collected effects from all sources
    effects: Vec<Effect>,
}

impl EffectCollector {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add effects from a single source (e.g., one attribute, one item, etc.)
    pub fn add_effects(&mut self, effects: &[Effect], current_time: &GameTimeStep) {
        self.effects.extend(
            effects.iter()
                .filter(|e| e.is_active(current_time))
                .cloned()
        );
    }
    
    /// Calculate the final effective value for any property
    /// Works generically - no hardcoded property handling!
    pub fn calculate_effective_value(&self, property: &EffectProperty, base_value: u64) -> u64 {
        // Filter effects for this property and sort by priority
        let mut applicable_effects: Vec<_> = self.effects.iter()
            .filter(|e| &e.property == property)
            .collect();
        applicable_effects.sort_by_key(|e| e.priority);
        
        // Apply stacking rules and modifications
        self.apply_effects_with_stacking(applicable_effects, base_value)
    }
    
    /// Apply effects respecting stacking behavior
    fn apply_effects_with_stacking(&self, effects: Vec<&Effect>, base_value: u64) -> u64 {
        if effects.is_empty() {
            return base_value;
        }
        
        // Group by stacking behavior
        let mut result = base_value;
        let mut additive_total: i64 = 0;
        let mut multiplicative_product = Percentage::ONE_HUNDRED; // Start at 100%
        let mut highest_value: Option<u64> = None;
        let mut set_values: Vec<u64> = Vec::new();
        
        for effect in effects {
            match (&effect.modification, &effect.stacking) {
                (EffectModification::Additive(amount), EffectStacking::Stack) => {
                    additive_total += amount;
                },
                (EffectModification::Additive(amount), EffectStacking::StackHighest) => {
                    let value = base_value.saturating_add_signed(*amount);
                    highest_value = Some(highest_value.map_or(value, |h| h.max(value)));
                },
                (EffectModification::Multiplicative(percentage), EffectStacking::Stack) => {
                    // Stack multiplicatively: combine percentages
                    // Example: 150% * 120% = (150 * 120) / 100 = 180%
                    multiplicative_product = multiplicative_product.multiply(*percentage);
                },
                (EffectModification::SetValue(value), _) => {
                    set_values.push(*value);
                },
                (EffectModification::PercentageOfBase(percentage), EffectStacking::Stack) => {
                    additive_total += percentage.apply_to(base_value) as i64;
                },
                (_, EffectStacking::Replace) => {
                    // For replace, just apply the last one
                    result = effect.apply_modification(result);
                },
                _ => {
                    // Default behavior
                    result = effect.apply_modification(result);
                }
            }
        }
        
        // Apply in order: SetValue > Additive > Multiplicative
        if let Some(set_value) = set_values.last() {
            result = *set_value;
        }
        
        if let Some(highest) = highest_value {
            result = highest;
        } else {
            result = result.saturating_add_signed(additive_total);
        }
        
        // Apply multiplicative effects using deterministic integer math
        result = multiplicative_product.apply_to(result);
        
        result
    }
    
    /// Get all unique properties that have effects
    pub fn get_affected_properties(&self) -> Vec<EffectProperty> {
        let mut properties: Vec<_> = self.effects.iter()
            .map(|e| e.property.clone())
            .collect();
        properties.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
        properties.dedup();
        properties
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EffectModification {
    /// Add/subtract fixed amount
    Additive(i64),
    /// Multiply by percentage (150% = 1.5x multiplier, 80% = 0.8x multiplier)
    /// Example: base 100, Multiplicative(150%) = 150
    Multiplicative(Percentage),
    /// Set to specific value
    SetValue(u64),
    /// Add percentage of base value
    /// Example: base 100, PercentageOfBase(25%) = 100 + 25 = 125
    PercentageOfBase(Percentage),
    /// Custom modification logic
    Custom { modification_type: String, data: serde_json::Value },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EffectDuration {
    /// Effect lasts forever until manually removed
    Permanent,
    /// Effect expires at specific game time
    UntilGameTime(GameTimeStep),
    /// Effect lasts for number of game steps from application
    Duration(u64),
    /// Effect lasts until specific condition met
    UntilCondition(EffectCondition),
    /// Custom duration logic
    Custom { duration_type: String, data: serde_json::Value },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EffectCondition {
    /// Always active
    Always,
    /// Active only during specific game time range
    GameTimeRange { start: GameTimeStep, end: GameTimeStep },
    /// Active when attribute meets requirement
    AttributeRequirement { attribute_definition_id: Uuid, min_value: u32 },
    /// Active when affinity mastery meets requirement
    AffinityRequirement { affinity_definition_id: Uuid, min_mastery: u32 },
    /// Custom condition logic
    Custom { condition_type: String, data: serde_json::Value },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EffectStacking {
    /// Only one instance can exist, new replaces old
    Replace,
    /// Multiple instances add together
    Stack,
    /// Only highest value applies
    StackHighest,
    /// Only most recent applies
    StackNewest,
    /// Custom stacking behavior
    Custom { stacking_type: String },
}


```

### **Effect System Integration**

```rust
/// Trait for any object that can have effects applied to it (context-aware)
pub trait Effectable {
    fn get_effects(&self) -> &Vec<Effect>;
    fn get_effects_mut(&mut self) -> &mut Vec<Effect>;
    
    fn add_effect(&mut self, effect: Effect) -> Result<(), EffectError>;
    fn remove_effect(&mut self, effect_id: &Uuid) -> bool;
    fn remove_expired_effects(&mut self, current_time: &GameTimeStep);
    
    /// Calculate effective value for a specific property considering all active effects
    /// Context-aware: the meaning of the property depends on the implementing object
    fn calculate_effective_property_value(&self, property: EffectProperty, base_value: u64, current_time: &GameTimeStep) -> u64;
    
    /// Get all effects that modify a specific property
    fn get_effects_for_property(&self, property: EffectProperty) -> Vec<&Effect>;
}

/// Trait for game object instances that can retrieve their definition
/// Provides type-safe access to definition data from instance objects
pub trait HasDefinition<T> {
    /// Get the UUID of the definition this instance is based on
    fn definition_id(&self) -> Uuid;
    
    /// Retrieve the definition object from the game data loader
    /// Returns None if the definition is not found (should never happen with valid data)
    fn get_definition<'a>(&self, loader: &'a GameDefinitionsLoader) -> Option<&'a T>;
}

// Example implementations for kickoff phase instances:

impl HasDefinition<AffinityDefinition> for AffinityInstance {
    fn definition_id(&self) -> Uuid {
        self.definition_id
    }
    
    fn get_definition<'a>(&self, loader: &'a GameDefinitionsLoader) -> Option<&'a AffinityDefinition> {
        loader.get_affinity_definition(&self.definition_id)
    }
}

impl HasDefinition<AttributeDefinition> for AttributeInstance {
    fn definition_id(&self) -> Uuid {
        self.definition_id
    }
    
    fn get_definition<'a>(&self, loader: &'a GameDefinitionsLoader) -> Option<&'a AttributeDefinition> {
        loader.get_attribute_definition(&self.definition_id)
    }
}

impl HasDefinition<EffectDefinition> for Effect {
    fn definition_id(&self) -> Uuid {
        self.definition_id
    }
    
    fn get_definition<'a>(&self, loader: &'a GameDefinitionsLoader) -> Option<&'a EffectDefinition> {
        loader.get_effect_definition(&self.definition_id)
    }
}

// Additional factory methods for Effect:
impl Effect {
    /// Create new effect instance from definition (generates fresh UUID)
    /// Copies duration and conditions from definition into the instance
    pub fn new_from_definition(
        definition_id: Uuid,
        source_id: Uuid,
        applied_at: GameTimeStep,
        loader: &GameDefinitionsLoader,
    ) -> Result<Self, CreationError> {
        let definition = loader.get_effect_definition(&definition_id)
            .ok_or(CreationError::DefinitionNotFound(definition_id))?;
        
        Ok(Self {
            id: Uuid::new_v4(),              // ✅ Explicit new UUID for fresh instance
            definition_id,                    // ✅ From loaded definition
            source_id,                        // What caused this effect
            applied_at,                       // ⏰ Game time when applied
            duration: definition.duration.clone(), // Copy duration from definition (instance tracks lifetime)
            conditions: definition.conditions.clone(), // Copy conditions from definition (can add more)
        })
    }
    
    /// Create effect with custom duration (different from definition default)
    pub fn new_with_custom_duration(
        definition_id: Uuid,
        source_id: Uuid,
        applied_at: GameTimeStep,
        duration: EffectDuration,
        loader: &GameDefinitionsLoader,
    ) -> Result<Self, CreationError> {
        let definition = loader.get_effect_definition(&definition_id)
            .ok_or(CreationError::DefinitionNotFound(definition_id))?;
        
        Ok(Self {
            id: Uuid::new_v4(),
            definition_id,
            source_id,
            applied_at,
            duration,                         // Custom duration instead of definition default
            conditions: definition.conditions.clone(),
        })
    }
    
    /// Create effect with additional conditions beyond definition defaults
    pub fn new_with_additional_conditions(
        definition_id: Uuid,
        source_id: Uuid,
        applied_at: GameTimeStep,
        additional_conditions: Vec<EffectCondition>,
        loader: &GameDefinitionsLoader,
    ) -> Result<Self, CreationError> {
        let definition = loader.get_effect_definition(&definition_id)
            .ok_or(CreationError::DefinitionNotFound(definition_id))?;
        
        let mut conditions = definition.conditions.clone();
        conditions.extend(additional_conditions);
        
        Ok(Self {
            id: Uuid::new_v4(),
            definition_id,
            source_id,
            applied_at,
            duration: definition.duration.clone(),
            conditions,                       // Definition conditions + additional ones
        })
    }
    
    /// Load effect instance from save data (preserves UUID and instance state)
    pub fn from_save_data(
        id: Uuid,                             // ✅ UUID from save file
        definition_id: Uuid,                  // ✅ Definition UUID
        source_id: Uuid,
        applied_at: GameTimeStep,
        duration: EffectDuration,             // Instance's current duration state
        conditions: Vec<EffectCondition>,     // Instance's current conditions
    ) -> Self {
        Self {
            id,                              // ✅ Preserved from save
            definition_id,
            source_id,
            applied_at,
            duration,                        // Restore instance duration state
            conditions,                      // Restore instance conditions
        }
    }
}

// Usage example:
// let affinity_instance: AffinityInstance = /* ... */;
// let loader: GameDefinitionsLoader = /* ... */;
// if let Some(definition) = affinity_instance.get_definition(&loader) {
//     println!("Affinity name: {}", definition.name);
//     println!("Description: {}", definition.description);
//     // Access any definition data without manual lookups
// }
```

## ⏰ **CRITICAL: Game Time System**

### **Game Time vs Real Time Separation** ⚠️ **MANDATORY**

**Game Time**: 
- ✅ **Step-based progression** - Time advances by discrete steps, not real-world seconds
- ✅ **Loop-aware** - Time resets to zero at start of each timeloop
- ✅ **Deterministic** - Same actions always take same time steps
- ✅ **Save/Load stable** - Game time preserved across sessions

**Real Time**:
- ✅ **Metadata only** - Used for creation timestamps, save times, etc.
- ✅ **Not gameplay-affecting** - Never influences game mechanics
- ✅ **Player convenience** - Shows when save was created, etc.

### **Game Time Implementation** 🕐

```rust
/// Game time measured in discrete steps, independent of real-world time
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GameTimeStep {
    /// Absolute step count from start of current timeloop
    Step(u64),
    /// Future expansion: Step within specific timeloop
    LoopStep { loop_number: u32, step: u64 },
    /// Future expansion: Relative time (e.g., "5 steps from now")
    Relative(i64),
}

impl GameTimeStep {
    pub fn zero() -> Self {
        GameTimeStep::Step(0)
    }
    
    pub fn new(step: u64) -> Self {
        GameTimeStep::Step(step)
    }
    
    pub fn advance(&mut self, steps: u64) {
        match self {
            GameTimeStep::Step(current) => *current += steps,
            // Future: handle other variants
            _ => {} // Placeholder for future variants
        }
    }
    
    pub fn next(&self) -> Self {
        match self {
            GameTimeStep::Step(step) => GameTimeStep::Step(step + 1),
            GameTimeStep::LoopStep { loop_number, step } => {
                GameTimeStep::LoopStep { loop_number: *loop_number, step: step + 1 }
            },
            GameTimeStep::Relative(offset) => GameTimeStep::Relative(offset + 1),
        }
    }
    
    pub fn add_steps(&self, steps: u64) -> Self {
        match self {
            GameTimeStep::Step(current_step) => GameTimeStep::Step(current_step + steps),
            GameTimeStep::LoopStep { loop_number, step } => {
                GameTimeStep::LoopStep { loop_number: *loop_number, step: step + steps }
            },
            GameTimeStep::Relative(offset) => GameTimeStep::Relative(offset + steps as i64),
        }
    }
    
    pub fn current_step(&self) -> u64 {
        match self {
            GameTimeStep::Step(step) => *step,
            GameTimeStep::LoopStep { step, .. } => *step,
            GameTimeStep::Relative(_) => panic!("Cannot get current step from relative time"),
        }
    }
}
```

### **Time Usage Rules** 🔒

```rust
// ✅ CORRECT - Game time for gameplay mechanics
pub struct Effect {
    pub effect_id: Uuid,
    pub source_id: Uuid,
    pub property: EffectProperty,
    pub modification: EffectModification,
    pub duration: EffectDuration,
    pub applied_at: GameTimeStep,         // ✅ Game time - affects mechanics
    pub description: String,
}

// ✅ CORRECT - Game time for training history  
pub struct AttributeTraining {
    pub trained_at: GameTimeStep,         // ✅ Game time - when in game this happened
    pub new_value: u32,
    pub training_method: String,
}

// ❌ WRONG - Don't use real-world time for gameplay
// pub expires_at: DateTime<Utc>,        // Would break on save/load across time zones
// pub created_at: DateTime<Utc>,        // Not needed for game mechanics
```

## 🎯 **CRITICAL: Deterministic Percentage Type**

### **Percentage - Deterministic Ordinal Type** ⚠️ **MANDATORY**

**Design Philosophy**: Eliminate floating point arithmetic from game mechanics to ensure perfect determinism and reproducibility across all platforms and game sessions.

```rust
/// Deterministic percentage type using whole number percentages
/// Minimum unit: 1% (no decimal precision needed for game mechanics)
/// Range: 0 to u16::MAX (0% to 65535% - no ceiling for multiplier stacking)
/// Examples: 1 = 1%, 50 = 50%, 100 = 100%, 150 = 150% (1.5x multiplier), 200 = 200% (2x multiplier)
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
    pub const ZERO: Percentage = Percentage(0);        // 0%
    pub const TEN: Percentage = Percentage(10);        // 10%
    pub const TWENTY_FIVE: Percentage = Percentage(25); // 25%
    pub const FIFTY: Percentage = Percentage(50);      // 50%
    pub const SEVENTY_FIVE: Percentage = Percentage(75); // 75%
    pub const ONE_HUNDRED: Percentage = Percentage(100); // 100%
    pub const ONE_TWENTY_FIVE: Percentage = Percentage(125); // 125%
    pub const ONE_FIFTY: Percentage = Percentage(150); // 150% (1.5x multiplier)
    pub const TWO_HUNDRED: Percentage = Percentage(200); // 200% (2x multiplier)
}

// Usage examples:
// let bonus = Percentage::from(25);                 // 25% (using From trait)
// let bonus = Percentage::new(25);                  // 25% (explicit)
// let result = bonus.apply_to(100);                 // = 25
// let multiplier: Percentage = 150.into();          // 150% (using Into trait)
// let value: u16 = multiplier.into();               // Extract raw value
// let stacked = Percentage::from(150).multiply(Percentage::from(120)); // 180%

// 🎯 BENEFITS OF PERCENTAGE TYPE:
// ✅ Perfect determinism - same calculations produce identical results every time
// ✅ Cross-platform consistency - no platform-specific floating point differences
// ✅ Save/Load stability - percentages serialize/deserialize identically
// ✅ Replay compatibility - recorded game sessions replay exactly
// ✅ Network sync - multiplayer games stay in sync without floating point drift
// ✅ Testing reliability - unit tests produce consistent results
// ✅ Performance - integer math is faster than floating point on most CPUs
// ✅ No ceiling - multipliers can stack infinitely for epic late-game effects
// ✅ Performance - integer math is faster than floating point on most CPUs
```

## 🚨 **CRITICAL: UUID Lifecycle Management**

### **UUID Source Authority** ⚠️ **MANDATORY**

**Definition UUIDs**: 
- ✅ **ALWAYS** come from external JSON files
- ✅ **NEVER** generated at runtime
- ✅ **IMMUTABLE** across all game sessions
- ✅ **SHARED** reference points for all instances

**Instance UUIDs**:
- ✅ **NEW UUID** when creating fresh instances (`Uuid::new_v4()`)
- ✅ **PRESERVE UUID** when loading from save files
- ⚠️ **NEVER** auto-generate in `Default` or `new()` constructors
- ⚠️ **EXPLICIT** UUID assignment required for all instance creation

### **UUID Lifecycle Rules** 🔒

```rust
// ❌ FORBIDDEN - Auto-generation in constructors
impl Default for SkillInstance {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(), // ❌ WRONG - Will break save/load
            // ...
        }
    }
}

// ✅ CORRECT - Explicit UUID management
impl SkillInstance {
    // For NEW instances (not from save file)
    pub fn new_from_definition(definition_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(), // ✅ Explicit new UUID
            definition_id,
            current_level: 1,
            // ...
        }
    }
    
    // For LOADED instances (from save file)
    pub fn from_save_data(
        id: Uuid,           // ✅ Preserved from save
        definition_id: Uuid,
        current_level: u32,
        // ... other saved fields
    ) -> Self {
        Self {
            id,             // ✅ Uses provided UUID from save
            definition_id,
            current_level,
            // ...
        }
    }
}
```

### **Critical Implementation Constraints** 🚨

1. **No Default Implementations**: Instance structs must NOT implement `Default` with auto-generated UUIDs
2. **Explicit Factory Methods**: Always use explicit factory methods for instance creation
3. **Save/Load UUID Preservation**: Serialization must preserve instance UUIDs exactly
4. **Definition UUID Immutability**: Definition UUIDs must never change between game sessions
5. **Runtime UUID Validation**: All instance definition_ids must reference valid loaded definitions

### **Project Structure** (Following TECHDESIGN.md + Data-Driven Extensions)
```
src/
├── main.rs                    # Application entry point
├── api/                       # REST endpoints & serialization
│   ├── mod.rs
│   ├── timelooper.rs         # Timelooper CRUD endpoints
│   ├── game_data.rs          # Game data CRUD endpoints
│   └── health.rs             # Health check endpoints
├── game_engine/              # Game flow & business logic (NO HARDCODED CONTENT)
│   ├── mod.rs
│   ├── creation.rs           # Character creation logic using loaded data
│   ├── state_manager.rs      # Game state management
│   ├── evaluator.rs          # Skill/requirement evaluation engine
│   └── loader.rs             # Game data loading and validation system
├── models/                   # Data structures only
│   ├── mod.rs
│   ├── timelooper.rs         # Timelooper entity with instance collections
│   ├── definitions/          # Immutable template definitions from external files
│   │   ├── mod.rs
│   │   ├── skill_definition.rs
│   │   ├── ability_definition.rs
│   │   ├── item_definition.rs
│   │   ├── affinity_definition.rs
│   │   └── card_definition.rs
│   ├── instances/            # Mutable objects with state and history
│   │   ├── mod.rs
│   │   ├── skill_instance.rs
│   │   ├── ability_instance.rs
│   │   ├── item_instance.rs
│   │   └── card_instance.rs
│   └── requirements.rs       # Requirement and evaluation system
└── storage/                  # Data persistence layer
    ├── mod.rs
    ├── repository.rs         # Generic repository pattern
    ├── file_storage.rs       # JSON file-based storage
    └── game_data_loader.rs   # External game data loading
```

### **External Game Data Structure**
```
game_data/
├── core/
│   ├── affinities.json       # All affinity definitions with UUIDs
│   ├── attributes.json       # Attribute definitions and constraints
│   ├── effects.json          # Effect definitions (bonuses, penalties, modifiers)
│   └── skills.json           # Skill definitions and progressions
├── content/
│   ├── cards/
│   │   ├── combat_cards.json
│   │   ├── event_cards.json
│   │   └── progress_cards.json
│   ├── events/
│   │   ├── world_events.json
│   │   └── character_events.json
│   ├── factions/
│   │   ├── guilds.json
│   │   └── houses.json
│   └── items/
│       ├── equipment.json
│       └── consumables.json
├── worlds/
│   ├── template_worlds.json  # World generation templates
│   └── locations.json        # Location definitions
└── metadata/
    ├── schema_version.json   # Data format versioning
    └── dependencies.json     # Cross-reference validation
```

### **Technology Stack**
- **Backend**: Rust + Axum (REST API)
- **Game Engine**: Pure evaluation engine (no hardcoded content)
- **Data Layer**: External JSON files with UUID cross-referencing
- **Frontend**: Bevy game engine (Future phase)
- **Serialization**: Serde JSON
- **Storage**: File-based JSON with UUID indexing
- **Testing**: Built-in Rust test framework

### **Game Engine Evaluation Philosophy**
The game engine maintains **the current timelooper state** and provides **standardized evaluation and instance management methods** but contains **zero hardcoded game content**:

```rust
// Engine CAN do (Single Timelooper Context):
engine.evaluate_skill_requirement(skill_definition_uuid, required_level);
engine.can_acquire_ability(ability_definition_uuid);
engine.train_skill(skill_instance_uuid, experience_points);
engine.use_ability(ability_instance_uuid, context);
engine.damage_item(item_instance_uuid, damage_amount);
engine.calculate_effective_attributes(); // Using current timelooper's instances
engine.get_skill_instances_by_definition(skill_definition_uuid);
engine.get_current_timelooper(); // Always available
engine.load_timelooper(timelooper_id); // Switch active timelooper (for loading saves)

// Engine CANNOT do (must come from external definitions):
// - Know what "Hand-to-Hand Combat" affinity does (comes from AffinityDefinition)
// - Define what skills exist or their effects (comes from SkillDefinition)
// - Determine card mechanics or abilities (comes from CardDefinition/AbilityDefinition)  
// - Hard-code any game content whatsoever
// - Create instances without referencing valid definitions
```

**Critical Separation Examples**:
- **Definition**: "Fireball Spell" can do 10-50 damage, requires 25 Magic skill, costs 10 mana
- **Instance 1**: Player's "Scorching Fireball" (same definition) - modified by enchantment, used 47 times, deals +5 damage
- **Instance 2**: Player's "Ice-tinged Fireball" (same definition) - cursed variant, deals cold damage instead
- **Multiple Ownership**: Player can have 3 "Iron Sword" instances, each with different durability and enchantments

## 🎯 **Single Timelooper Architecture Analysis**

### **Benefits of Single Timelooper Context** ✅

1. **Simplified API**: No need to pass timelooper to every engine method
2. **Cleaner Code**: `engine.train_skill(skill_id, xp)` vs `engine.train_skill(timelooper, skill_id, xp)`
3. **Game Design Alignment**: Matches the single-character timeloop concept perfectly
4. **State Consistency**: Engine always knows the active game state
5. **Simpler Testing**: Test methods don't need timelooper parameter setup
6. **Better Performance**: No repeated parameter passing or lookups

### **Design Considerations** ⚠️

1. **Save/Load Mechanism**: Engine needs clean timelooper switching for different saves
2. **Testing Isolation**: Tests need easy way to set up different timelooper states
3. **Future Multiplayer**: If ever needed, would require architectural changes (unlikely given game concept)

### **Mitigation Strategies** ✅

```rust
// Clean save/load switching
impl GameEngine {
    pub fn new_game(&mut self, name: String, affinity_id: Uuid) -> Result<(), GameError> {
        self.current_timelooper = Some(self.create_timelooper(name, affinity_id)?);
        self.session_id = Some(Uuid::new_v4());
        Ok(())
    }
    
    pub fn load_game(&mut self, save_id: Uuid) -> Result<(), GameError> {
        self.current_timelooper = Some(self.storage.read(&save_id)?);
        self.session_id = Some(Uuid::new_v4());
        Ok(())
    }
    
    // For testing - easy timelooper injection
    pub fn set_timelooper_for_testing(&mut self, timelooper: Timelooper) {
        self.current_timelooper = Some(timelooper);
    }
}
```

## 🚀 **Development Phases - Detailed Roadmap**

### **Phase 1: Foundation Setup** (Estimated: 2-3 hours)
**Goal**: Establish project structure and dependencies

#### **Step 1.1: Project Dependencies** ⭐ **MANDATORY**
- Add required dependencies to Cargo.toml
- Set up development and testing dependencies
- Configure project metadata

**Dependencies Required**:
```toml
[dependencies]
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
tokio = { version = "1.0", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "fs"] }
tracing = "0.1"
tracing-subscriber = "0.3"
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tempfile = "3.0"
```

**Quality Gates**:
- `cargo build` - 0 warnings
- `cargo clippy` - 0 suggestions
- Project structure created according to TECHDESIGN.md

#### **Step 1.2: Module Structure Setup** ⭐ **MANDATORY**
- Create module hierarchy according to architecture
- Set up mod.rs files with proper exports
- Establish testing framework structure

**Deliverables**:
- All module files created with basic structure
- Module exports configured
- Basic integration test setup

**Quality Gates**:
- `cargo build` passes cleanly
- `cargo test` runs successfully (even with empty tests)
- `cargo clippy` shows no warnings

### **Phase 2: Core Data Models & Game Data System** (Estimated: 4-5 hours)
**Goal**: Implement the data-driven architecture with external game definitions and UUID-based Timelooper structure

#### **Step 2.1: Definition vs Instance Architecture with UUID Lifecycle** ⭐ **MANDATORY**
- Define all external game entity **definitions** (templates from files)  
- Define all **instance** structures (mutable objects with state)
- Implement UUID-based cross-referencing between definitions and instances
- **CRITICAL**: Implement proper UUID lifecycle management for save/load integrity
- Create requirement and evaluation framework
- Add Serde serialization/deserialization for both definitions and instances

**Key Architectural Principles**:
- **Definitions**: Immutable, loaded from external files, shared templates, **UUIDs from files ONLY**
- **Instances**: Mutable, owned by game entities, individual state and history, **Explicit UUID management**
- **References**: Instances reference definitions via UUID, never embed them
- **UUID Lifecycle**: No auto-generation in constructors, explicit factory methods only

**Core External Definitions**:
```rust
// Requirement system for skills, abilities, events
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Requirement {
    pub requirement_type: RequirementType,
    pub target_id: Uuid,
    pub value: RequirementValue,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RequirementType {
    SkillLevel,      // Must have skill at level X
    AttributeLevel,  // Must have attribute at level X
    HasAbility,      // Must possess specific ability
    HasCard,         // Must own specific card
    MoneyAmount,     // Must have minimum money
    Status,          // Must have status level with faction
    Custom(String),  // Custom evaluation logic
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RequirementValue {
    Integer(u32),
    Boolean(bool),
    String(String),
    List(Vec<Uuid>),
}
```

**Timelooper Structure** (Simplified for Kickoff Phase):
```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Timelooper {
    pub id: Uuid,                             // ⚠️ CRITICAL: Never auto-generated in constructors
    pub name: String,                         // Timelooper's chosen name
    pub affinities: Vec<AffinityInstance>,    // Instantiated affinities (can have multiple)
    pub attributes: HashMap<Uuid, AttributeInstance>, // Attribute instances (Physical, Mental)
    pub base_money: u64,                      // Base money amount (without effects)
    pub effects: Vec<Effect>,                 // 🎯 Global effects affecting the timelooper
    pub current_game_time: GameTimeStep,      // ⏰ Current game time step
    pub loop_count: u32,                      // Number of loops completed
}

impl Timelooper {
    /// Calculate current effective money considering effects from ALL sources:
    /// - Direct effects on timelooper
    /// - Effects from attributes, affinities, items, etc. automatically included
    pub fn current_money(&self) -> u64 {
        let mut collector = EffectCollector::new();
        self.collect_effects(&mut collector, &self.current_game_time);
        collector.calculate_effective_value(&EffectProperty::PrimaryValue, self.base_money)
    }
    
    /// Get current effective Physical attribute value (aggregated from all sources)
    pub fn current_physical(&self) -> u32 {
        if let Some(physical_attr) = self.attributes.values().find(|a| a.is_physical()) {
            let mut collector = EffectCollector::new();
            
            // Collect from attribute and all related sources
            physical_attr.collect_effects(&mut collector, &self.current_game_time);
            
            // Collect from affinities that boost Physical
            for affinity in &self.affinities {
                affinity.collect_effects(&mut collector, &self.current_game_time);
            }
            
            // Future: Collect from equipped items, active abilities, etc.
            
            collector.calculate_effective_value(&EffectProperty::PrimaryValue, physical_attr.base_value as u64) as u32
        } else {
            0
        }
    }
    
    /// Advance game time by one step
    pub fn advance_time(&mut self) {
        self.current_game_time = self.current_game_time.next();
        
        // Clean up expired effects
        self.remove_expired_effects();
    }
    
    /// Remove all expired effects from timelooper and all instances
    pub fn remove_expired_effects(&mut self) {
        // Remove timelooper effects
        self.effects.retain(|effect| effect.is_active(&self.current_game_time));
        
        // Remove affinity effects  
        for affinity in &mut self.affinities {
            affinity.effects.retain(|effect| effect.is_active(&self.current_game_time));
        }
        
        // Remove attribute effects
        for attribute in self.attributes.values_mut() {
            attribute.effects.retain(|effect| effect.is_active(&self.current_game_time));
        }
    }
}

impl EffectAggregator for Timelooper {
    /// Collect effects from timelooper and ALL owned objects
    fn collect_effects(&self, collector: &mut EffectCollector, current_time: &GameTimeStep) {
        // Add timelooper's own effects
        collector.add_effects(&self.effects, current_time);
        
        // Collect from all owned objects
        for affinity in &self.affinities {
            affinity.collect_effects(collector, current_time);
        }
        
        for attribute in self.attributes.values() {
            attribute.collect_effects(collector, current_time);
        }
        
        // Future: Collect from items, skills, abilities, cards, etc.
        // for item in &self.inventory {
        //     item.collect_effects(collector, current_time);
        // }
        // for skill in self.skills.values() {
        //     skill.collect_effects(collector, current_time);
        // }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AffinityInstance {
    pub id: Uuid,                             // ⚠️ CRITICAL: Explicit UUID management
    pub definition_id: Uuid,                  // ✅ References AffinityDefinition
    pub acquired_at: GameTimeStep,            // ⏰ Game time when this affinity was gained
    pub base_mastery_level: u32,              // Base mastery level (without effects)
    pub effects: Vec<Effect>,                 // 🎯 All effects affecting this affinity
    pub mastery_history: Vec<MasteryProgress>, // History of mastery improvements
}

impl AffinityInstance {
    /// Calculate current effective mastery level considering all active effects
    pub fn current_mastery_level(&self, current_time: &GameTimeStep) -> u32 {
        let mut collector = EffectCollector::new();
        self.collect_effects(&mut collector, current_time);
        collector.calculate_effective_value(&EffectProperty::PrimaryValue, self.base_mastery_level as u64) as u32
    }
    
    /// Get mastery training speed multiplier from effects (deterministic)
    /// Returns percentage where 100% = normal speed, 150% = 1.5x faster, 50% = 0.5x slower
    pub fn mastery_speed_multiplier(&self, current_time: &GameTimeStep) -> Percentage {
        let mut collector = EffectCollector::new();
        self.collect_effects(&mut collector, current_time);
        let base_speed = Percentage::ONE_HUNDRED.get() as u64; // 100%
        let result = collector.calculate_effective_value(&EffectProperty::TrainingSpeed, base_speed);
        Percentage::from(result.min(u16::MAX as u64) as u16)
    }
}

impl EffectAggregator for AffinityInstance {
    fn collect_effects(&self, collector: &mut EffectCollector, current_time: &GameTimeStep) {
        collector.add_effects(&self.effects, current_time);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttributeInstance {
    pub id: Uuid,                             // ⚠️ CRITICAL: Explicit UUID management
    pub definition_id: Uuid,                  // ✅ References AttributeDefinition (Physical/Mental)
    pub base_value: u32,                      // Base attribute value (without effects)
    pub effects: Vec<Effect>,                 // 🎯 All effects affecting this attribute
    pub training_history: Vec<AttributeTraining>,    // History of attribute improvements
}

impl AttributeInstance {
    /// Calculate current effective value considering all active effects
    pub fn current_value(&self, current_time: &GameTimeStep) -> u32 {
        let mut collector = EffectCollector::new();
        self.collect_effects(&mut collector, current_time);
        collector.calculate_effective_value(&EffectProperty::PrimaryValue, self.base_value as u64) as u32
    }
    
    /// Get training speed multiplier from effects (deterministic)
    /// Returns percentage where 100% = normal speed, 150% = 1.5x faster, 50% = 0.5x slower
    pub fn training_speed_multiplier(&self, current_time: &GameTimeStep) -> Percentage {
        let mut collector = EffectCollector::new();
        self.collect_effects(&mut collector, current_time);
        let base_speed = Percentage::ONE_HUNDRED.get() as u64; // 100%
        let result = collector.calculate_effective_value(&EffectProperty::TrainingSpeed, base_speed);
        Percentage::from(result.min(u16::MAX as u64) as u16)
    }
    
    /// Helper to check if this is the Physical attribute (based on definition lookup)
    /// In real implementation, would check against loaded AttributeDefinition
    pub fn is_physical(&self) -> bool {
        // Placeholder - in real code would lookup definition and check name
        // For now, assume first attribute in sorted order is Physical
        true // TODO: implement proper definition lookup
    }
}

impl EffectAggregator for AttributeInstance {
    fn collect_effects(&self, collector: &mut EffectCollector, current_time: &GameTimeStep) {
        collector.add_effects(&self.effects, current_time);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttributeTraining {
    pub trained_at: GameTimeStep,             // ⏰ Game time when training occurred
    pub new_value: u32,                       // New value after training
    pub training_method: String,              // How the training was accomplished
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MasteryProgress {
    pub achieved_at: GameTimeStep,            // ⏰ Game time when mastery level was reached
    pub new_level: u32,                       // New mastery level achieved
    pub method: String,                       // How mastery was improved
}

// 🎯 Context-Aware Effect System Examples (replaces old TemporaryModifier system)

impl Effect {
    /// Create a temporary attribute value bonus (e.g., from potion, training boost)  
    /// ADD THIS TO AttributeInstance.effects to affect that attribute's value
    pub fn temporary_value_bonus(
        source_id: Uuid,
        bonus_amount: i64,
        duration_steps: u64,
        applied_at: GameTimeStep,
        description: String,
    ) -> Self {
        Self {
            effect_id: Uuid::new_v4(),
            source_id,
            property: EffectProperty::PrimaryValue, // Affects whatever the "primary value" is for the container
            modification: EffectModification::Additive(bonus_amount),
            duration: EffectDuration::Duration(duration_steps),
            conditions: vec![EffectCondition::Always],
            stacking: EffectStacking::Stack,
            priority: 0,
            applied_at,
            description,
        }
    }
    
    /// Create a training speed multiplier (e.g., from mastery, items)
    /// ADD THIS TO AttributeInstance.effects OR AffinityInstance.effects to affect training speed
    pub fn training_speed_multiplier(
        source_id: Uuid,
        multiplier: f64,
        duration: EffectDuration,
        applied_at: GameTimeStep,
        description: String,
    ) -> Self {
        Self {
            effect_id: Uuid::new_v4(),
            source_id,
            property: EffectProperty::TrainingSpeed,
            modification: EffectModification::Multiplicative(multiplier),
            duration,
            conditions: vec![EffectCondition::Always],
            stacking: EffectStacking::StackHighest, // Only best training bonus applies
            priority: 50,
            applied_at,
            description,
        }
    }
    
    /// Check if this effect is currently active based on duration and conditions
    pub fn is_active(&self, current_time: &GameTimeStep) -> bool {
        // Check duration first
        let duration_active = match &self.duration {
            EffectDuration::Permanent => true,
            EffectDuration::UntilGameTime(end_time) => current_time <= end_time,
            EffectDuration::Duration(steps) => {
                let end_time = self.applied_at.add_steps(*steps);
                current_time <= &end_time
            },
            EffectDuration::UntilCondition(_) => true, // TODO: implement condition checking
            EffectDuration::Custom { .. } => true, // TODO: implement custom logic
        };
        
        if !duration_active {
            return false;
        }
        
        // Check all conditions (all must be true)
        self.conditions.iter().all(|condition| {
            match condition {
                EffectCondition::Always => true,
                EffectCondition::GameTimeRange { start, end } => {
                    current_time >= start && current_time <= end
                },
                // TODO: implement other condition types when needed
                _ => true,
            }
        })
    }
    
    /// Apply this effect's modification to a base value (deterministic integer math)
    pub fn apply_modification(&self, base_value: u64) -> u64 {
        match &self.modification {
            EffectModification::Additive(amount) => {
                if *amount >= 0 {
                    base_value.saturating_add(*amount as u64)
                } else {
                    base_value.saturating_sub((-*amount) as u64)
                }
            },
            EffectModification::Multiplicative(percentage) => {
                percentage.apply_to(base_value)
            },
            EffectModification::SetValue(value) => *value,
            EffectModification::PercentageOfBase(percentage) => {
                base_value + percentage.apply_to(base_value)
            },
            EffectModification::Custom { .. } => base_value, // TODO: implement custom logic
        }
    }
}

// 🎯 GENERIC EFFECT AGGREGATION EXAMPLES:

/*
// Example 1: Calculate Physical attribute with effects from EVERYWHERE
let timelooper = /* ... loaded timelooper ... */;
let physical_value = timelooper.current_physical();

// Behind the scenes, the trait-based collector automatically gathers effects from:
// 1. The Physical attribute's own effects (training bonuses, temporary buffs)
// 2. All affinities that grant Physical bonuses (Hand-to-Hand Combat affinity)
// 3. All equipped items that boost Physical (Sword of Strength)
// 4. All active abilities that modify Physical (Berserker Rage)
// 5. All cards in play that affect Physical (Strength Training card)
// Each object implements EffectAggregator trait - no hardcoding!

// Example 2: Sword adds +5 Physical - stored on Physical attribute, automatically aggregated
let sword_effect = Effect::temporary_value_bonus(
    sword_item_id,
    5,
    1000,
    current_time,
    "Sword of Strength".to_string(),
);
timelooper.attributes[physical_id].effects.push(sword_effect);
// Next time current_physical() is called, the trait collects it automatically!

// Example 3: Training potion doubles training speed - automatically aggregates
let potion_effect = Effect::training_speed_multiplier(
    potion_id,
    2.0,
    EffectDuration::Duration(100),
    current_time,
    "Training Potion".to_string(),
);
// Add to specific attribute
timelooper.attributes[physical_id].effects.push(potion_effect.clone());
// Or add to timelooper for global effect
timelooper.effects.push(potion_effect);
// Both work! Trait implementation collects from both sources

// Example 4: Generic aggregation for ANY property using the trait
let mut collector = EffectCollector::new();

// Collect via trait - each object knows how to contribute its effects
timelooper.collect_effects(&mut collector, &current_time);
// This automatically calls collect_effects on all owned objects!

// Calculate ANY property - no hardcoding! All using deterministic integer math!
let effective_money = collector.calculate_effective_value(&EffectProperty::PrimaryValue, base_money);
let training_speed_value = collector.calculate_effective_value(&EffectProperty::TrainingSpeed, Percentage::ONE_HUNDRED.get() as u64);
let training_speed: Percentage = (training_speed_value as u16).into();
let time_modifier = collector.calculate_effective_value(&EffectProperty::TimeModifier, base_time);

// Example 5: Adding new game objects - just implement the trait!
impl EffectAggregator for QuestInstance {
    fn collect_effects(&self, collector: &mut EffectCollector, current_time: &GameTimeStep) {
        collector.add_effects(&self.effects, current_time);
        // Could also collect from quest-owned objects if needed
    }
}
// Now quests automatically participate in effect aggregation!

// The beauty: 
// - Trait-based composition - each object contributes independently
// - No property-specific code paths
// - Add new properties by just adding to EffectProperty enum
// - Add new game objects by implementing EffectAggregator trait
// - Effects automatically bubble up through trait calls
// - Stacking rules applied consistently everywhere
// - Easy to test - mock any trait implementation
*/

// 🔮 FUTURE EXPANSIONS (Not implemented in kickoff phase):
// - skills: HashMap<Uuid, SkillInstance>
// - abilities: Vec<AbilityInstance>  
// - cards: Vec<CardInstance>
// - inventory: Vec<ItemInstance>
// - equipment: HashMap<Uuid, ItemInstance>
// - memories: Vec<Memory>
// - status: HashMap<Uuid, StatusLevel>
// - renown: u32
// - assets: Vec<AssetInstance>
```
```

**UUID Lifecycle Implementation Examples** (Kickoff Phase Focus):
```rust
// ⚠️ UUID LIFECYCLE IMPLEMENTATION - AffinityInstance
impl AffinityInstance {
    /// Create new affinity instance (generates fresh UUID)
    pub fn new_from_definition(definition_id: Uuid, current_game_time: GameTimeStep) -> Self {
        Self {
            id: Uuid::new_v4(),          // ✅ Explicit new UUID for fresh instance
            definition_id,               // ✅ From loaded definition
            acquired_at: current_game_time, // ⏰ Game time when acquired
            base_mastery_level: 0,       // Starting mastery (without effects)
            effects: Vec::new(),         // 🎯 No effects initially
            mastery_history: Vec::new(),
        }
    }
    
    /// Load affinity instance from save data (preserves UUID)
    pub fn from_save_data(
        id: Uuid,                   // ✅ UUID from save file
        definition_id: Uuid,        // ✅ Definition UUID (must exist in loaded definitions)
        acquired_at: GameTimeStep,  // ⏰ Game time from save
        base_mastery_level: u32,
        effects: Vec<Effect>,       // 🎯 Effects from save data
        mastery_history: Vec<MasteryProgress>,
    ) -> Self {
        Self {
            id,                     // ✅ Preserved from save
            definition_id,          // ✅ Must validate against loaded definitions
            acquired_at,            // ⏰ Game time preserved
            base_mastery_level,
            effects,                // 🎯 Effects preserved
            mastery_history,
        }
    }
}

// ⚠️ UUID LIFECYCLE IMPLEMENTATION - AttributeInstance  
impl AttributeInstance {
    /// Create new attribute instance (generates fresh UUID)
    pub fn new_from_definition(definition_id: Uuid, base_value: u32) -> Self {
        Self {
            id: Uuid::new_v4(),          // ✅ Explicit new UUID for fresh instance
            definition_id,               // ✅ From loaded definition
            base_value,                  // Base value without effects
            effects: Vec::new(),         // 🎯 No effects initially
            training_history: Vec::new(),
        }
    }
    
    /// Load attribute instance from save data (preserves UUID)
    pub fn from_save_data(
        id: Uuid,                   // ✅ UUID from save file
        definition_id: Uuid,        // ✅ Definition UUID (must exist in loaded definitions)
        base_value: u32,
        effects: Vec<Effect>,       // 🎯 Effects from save data
        training_history: Vec<AttributeTraining>,
    ) -> Self {
        Self {
            id,                     // ✅ Preserved from save
            definition_id,          // ✅ Must validate against loaded definitions
            base_value,
            effects,                // 🎯 Effects preserved with game time
            training_history,       // ⏰ Contains game time for trained_at
        }
    }
}

// ❌ FORBIDDEN - No Default implementations that generate UUIDs
// impl Default for AffinityInstance { ... } // DO NOT IMPLEMENT
// impl Default for AttributeInstance { ... } // DO NOT IMPLEMENT

// 🔮 FUTURE INSTANCE TYPES (Not implemented in kickoff):
// - SkillInstance, AbilityInstance, ItemInstance, CardInstance, etc.
// - Will follow same UUID lifecycle patterns when added in later phases
```

**Definition Objects** (Simplified for Kickoff Phase):
```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AffinityDefinition {
    pub id: Uuid,                       // ✅ Definition identifier (from external file)
    pub name: String,                   // e.g., "Hand-to-Hand Combat", "Psionic", "Business"
    pub description: String,            // Short description
    pub lore_text: String,             // Background story/flavor text
    pub attribute_bonuses: HashMap<Uuid, u32>, // Physical/Mental attribute bonuses
    pub mastery_benefits: HashMap<u32, String>, // Mastery level -> benefit description
    pub cultural_background: String,    // Cultural/social context
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttributeDefinition {
    pub id: Uuid,                       // ✅ Definition identifier (from external file)  
    pub name: String,                   // "Physical" or "Mental"
    pub description: String,            // What this attribute represents
    pub min_value: u32,                 // Minimum possible value (usually 1)
    pub max_value: u32,                 // Maximum possible value (e.g., 100)
    pub default_value: u32,             // Starting value for new timeloopers
    pub training_difficulty: Percentage, // How hard it is to improve (100% = normal, 150% = harder, 50% = easier)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EffectDefinition {
    pub id: Uuid,                       // ✅ Definition identifier (from external file)
    pub name: String,                   // e.g., "Strength Bonus", "Training Speed Boost"
    pub description: String,            // Human-readable description of what this effect does
    pub property: EffectProperty,       // WHICH property this effect modifies (immutable - different property = different definition)
    pub modification: EffectModification, // HOW this effect modifies the property (immutable - different modification = different definition)
    pub duration: EffectDuration,       // Default duration template (copied to instances at creation)
    pub conditions: Vec<EffectCondition>, // Default conditions template (copied to instances, can have more added)
    pub stacking: EffectStacking,       // How multiple instances of this effect combine
    pub priority: i32,                  // Default application order
    pub icon: Option<String>,           // Optional icon identifier
    pub visual_effect: Option<String>,  // Optional visual effect identifier
}

// 📝 EFFECT DESIGN PHILOSOPHY:
// - Property & Modification are IMMUTABLE in definition - they define WHAT this effect IS
//   → Need different property? Create a new EffectDefinition (e.g., "Physical Bonus" vs "Mental Bonus")
//   → Need different modification? Create a new EffectDefinition (e.g., "+5 bonus" vs "+10 bonus")
// - Duration is COPIED to instance at creation - the instance tracks its own lifetime independently
//   → Instance duration can be customized at creation time (e.g., potion lasts 50 steps vs default 100)
//   → Duration changes as game progresses (expires, gets extended, etc.)
// - Conditions are COPIED to instance at creation - instances can have additional conditions added
//   → Start with definition's base conditions, add instance-specific ones if needed
//   → Example: Definition has "Always" condition, instance adds "Only during combat" condition

// 🔮 FUTURE DEFINITION TYPES (Not implemented in kickoff phase):
// These will be added in subsequent development phases

/*
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SkillDefinition {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    // ... to be implemented in Phase 7+
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AbilityDefinition {
    pub id: Uuid,
    pub name: String,  
    pub description: String,
    // ... to be implemented in Phase 7+
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ItemDefinition {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    // ... to be implemented in Phase 7+
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CardDefinition {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    // ... to be implemented in Phase 7+
}
*/
```

**Quality Gates**:
- 100% test coverage for all definitions AND instances
- Clear separation between definition and instance tested
- Instance creation from definitions tested
- Multiple instances from same definition tested
- Instance state modification tested without affecting definitions
- **UUID lifecycle management tested** (new creation vs save/load scenarios)
- **Save/load UUID preservation verified** (round-trip testing)
- **Definition UUID immutability validated** (same UUIDs across game sessions)
- **No auto-UUID generation in constructors enforced** (compilation/linting checks)
- Zero clippy warnings

#### **Step 2.2: Game Data Loader System** ⭐ **MANDATORY**  
- Implement definition loading from external JSON files (definitions only)
- Create validation system for cross-references between definitions
- Add instance factory methods for creating instances from definitions
- Add startup initialization and data integrity checking

**Game Data Loader Features** (Simplified for Kickoff):
```rust
pub struct GameDefinitionsLoader {
    pub affinities: HashMap<Uuid, AffinityDefinition>,  // ✅ UUIDs from external files
    pub attributes: HashMap<Uuid, AttributeDefinition>, // ✅ Physical, Mental definitions
    pub effects: HashMap<Uuid, EffectDefinition>,       // ✅ Effect templates (bonuses, penalties, etc.)
    // 🔮 Future additions:
    // pub skills: HashMap<Uuid, SkillDefinition>,
    // pub abilities: HashMap<Uuid, AbilityDefinition>,   
    // pub cards: HashMap<Uuid, CardDefinition>,
    // pub items: HashMap<Uuid, ItemDefinition>,
}

impl GameDefinitionsLoader {
    pub async fn load_all_definitions(data_path: &Path) -> Result<Self, LoadError>;
    pub fn validate_definition_references(&self) -> Result<(), ValidationError>;
    
    // Definition access methods (kickoff phase)
    pub fn get_affinity_definition(&self, id: &Uuid) -> Option<&AffinityDefinition>;
    pub fn get_attribute_definition(&self, id: &Uuid) -> Option<&AttributeDefinition>;
    pub fn get_effect_definition(&self, id: &Uuid) -> Option<&EffectDefinition>;
    pub fn list_all_affinities(&self) -> Vec<&AffinityDefinition>;
    pub fn list_all_attributes(&self) -> Vec<&AttributeDefinition>;
    pub fn list_all_effects(&self) -> Vec<&EffectDefinition>;
    
    // ✅ Instance factory methods (generate NEW UUIDs for fresh instances)
    pub fn create_affinity_instance(&self, definition_id: &Uuid) -> Result<AffinityInstance, CreationError> {
        let _definition = self.get_affinity_definition(definition_id)
            .ok_or(CreationError::DefinitionNotFound(*definition_id))?;
        Ok(AffinityInstance::new_from_definition(*definition_id)) // ✅ Generates new UUID
    }
    
    pub fn create_attribute_instance(&self, definition_id: &Uuid) -> Result<AttributeInstance, CreationError> {
        let definition = self.get_attribute_definition(definition_id)
            .ok_or(CreationError::DefinitionNotFound(*definition_id))?;
        Ok(AttributeInstance::new_from_definition(*definition_id, definition.default_value))
    }
    
    pub fn create_effect_instance(
        &self,
        definition_id: &Uuid,
        source_id: Uuid,
        applied_at: GameTimeStep,
    ) -> Result<Effect, CreationError> {
        Effect::new_from_definition(*definition_id, source_id, applied_at, self)
    }
    
    // ✅ Validation that instance definition_ids exist in loaded definitions
    pub fn validate_timelooper_references(&self, timelooper: &Timelooper) -> Result<(), ValidationError>;
    
    // 🔮 Future instance factory methods:
    // pub fn create_skill_instance(&self, definition_id: &Uuid) -> Result<SkillInstance, CreationError>;
    // pub fn create_ability_instance(&self, definition_id: &Uuid, context: &str) -> Result<AbilityInstance, CreationError>;
    // pub fn create_item_instance(&self, definition_id: &Uuid) -> Result<ItemInstance, CreationError>;
}
```

**Quality Gates**:
- 100% test coverage for data loading and validation
- Cross-reference validation tested with invalid UUIDs
- Performance tested with large datasets
- Error handling for malformed JSON files

#### **Step 2.3: Instance-Based Timelooper Implementation** ⭐ **MANDATORY**
- Implement Timelooper struct with instance collections (not just UUIDs)
- Create validation that ensures all instance definition_ids exist in loaded definitions
- Add helper methods for instance management and operations
- Implement instance lifecycle management (creation, modification, removal)

**Instance Management Features**:
```rust
impl Timelooper {
    // Instance creation from definitions
    pub fn add_skill_from_definition(&mut self, definition_id: &Uuid, loader: &GameDefinitionsLoader) -> Result<Uuid, Error>;
    pub fn acquire_ability_from_definition(&mut self, definition_id: &Uuid, context: &str, loader: &GameDefinitionsLoader) -> Result<Uuid, Error>;
    pub fn add_item_from_definition(&mut self, definition_id: &Uuid, loader: &GameDefinitionsLoader) -> Result<Uuid, Error>;
    
    // Instance queries and operations
    pub fn get_skill_instance(&self, instance_id: &Uuid) -> Option<&SkillInstance>;
    pub fn get_skill_level(&self, definition_id: &Uuid) -> u32;  // Highest level if multiple instances
    pub fn has_ability(&self, definition_id: &Uuid) -> bool;
    pub fn get_items_by_definition(&self, definition_id: &Uuid) -> Vec<&ItemInstance>;
    
    // Instance modifications
    pub fn train_skill(&mut self, instance_id: &Uuid, experience: u32) -> Result<bool, Error>; // returns true if level up
    pub fn use_ability(&mut self, instance_id: &Uuid) -> Result<(), Error>;
    pub fn damage_item(&mut self, instance_id: &Uuid, damage: u8) -> Result<(), Error>;
}
```

**Validation with Loaded Definitions**:
- Name: 3-50 characters, alphanumeric + spaces
- Affinity ID: Must exist in loaded affinity definitions
- All instance definition_ids must exist in corresponding definition collections
- Instance state must be valid according to definition constraints
- Multiple instances of same definition allowed and properly tracked

**Quality Gates**:
- 100% test coverage for instance creation, modification, and queries
- Multiple instances of same definition tested
- Instance state changes tested without affecting definitions
- Invalid definition_id reference handling tested
- Instance lifecycle (create->modify->delete) fully tested

### **Phase 3: Storage Layer Implementation with UUID Preservation** (Estimated: 2-3 hours)
**Goal**: Create robust, UUID-indexed JSON persistence with UUID lifecycle preservation

#### **Step 3.1: Generic Repository Pattern** ⭐ **MANDATORY**
- Implement trait-based repository interface
- Support CRUD operations with UUID keys
- Abstract storage implementation details

```rust
pub trait Repository<T> {
    async fn create(&self, entity: &T) -> Result<(), StorageError>;
    async fn read(&self, id: &Uuid) -> Result<Option<T>, StorageError>;
    async fn update(&self, id: &Uuid, entity: &T) -> Result<(), StorageError>;
    async fn delete(&self, id: &Uuid) -> Result<bool, StorageError>;
    async fn list_all(&self) -> Result<Vec<(Uuid, T)>, StorageError>;
}
```

**Quality Gates**:
- 100% test coverage using in-memory implementation
- All error scenarios tested
- Thread-safety considerations addressed
- **UUID preservation tested** (save -> load -> verify same UUIDs)
- **Round-trip serialization tested** (no UUID corruption)

#### **Step 3.2: File-Based Storage Implementation** ⭐ **MANDATORY**
- JSON file storage with UUID-based filenames
- Atomic write operations (temp file + rename)
- Directory management and file organization

**File Structure**:
```
saves/
├── timeloopers/
│   ├── {uuid1}.json
│   ├── {uuid2}.json
│   └── index.json
└── metadata/
    └── game_info.json
```

**Quality Gates**:
- 100% test coverage including file I/O edge cases
- Concurrent access handling tested
- Corruption recovery mechanisms tested
- Performance benchmarks for large datasets
- **UUID preservation across save/load cycles verified**
- **JSON serialization maintains exact UUID strings** (no formatting changes)
- **File-based storage preserves instance UUID uniqueness**

#### **Step 3.3: Error Handling and Recovery** ⭐ **MANDATORY**
- Comprehensive error type definitions
- Graceful handling of file system issues
- Data corruption detection and recovery

**Quality Gates**:
- All error paths tested
- Recovery scenarios validated
- User-friendly error messages

### **Phase 4: Data-Driven Game Engine Foundation** (Estimated: 3-4 hours)
**Goal**: Implement game engine that operates entirely on loaded external data with evaluation capabilities

#### **Step 4.1: Game Engine with Instance Management** ⭐ **MANDATORY**
- Game engine that operates on definitions and manages instances
- Instance creation, modification, and lifecycle management
- Requirement evaluation using both definitions and instance state
- State management with validation against loaded definitions

```rust
pub struct GameEngine {
    pub definitions: Arc<GameDefinitionsLoader>,
    pub storage: Arc<dyn Repository<Timelooper>>,
    pub current_timelooper: Option<Timelooper>,   // Always available when game is active
    pub world_state: WorldState,                  // Future expansion
    pub session_id: Option<Uuid>,
}

impl GameEngine {
    pub fn new(definitions: Arc<GameDefinitionsLoader>, storage: Arc<dyn Repository<Timelooper>>) -> Self;
    
    // Game time management
    pub fn current_game_time(&self) -> GameTimeStep;
    pub fn advance_time(&mut self, steps: u64) -> GameTimeStep;
    pub fn reset_time_for_new_loop(&mut self) -> u32; // Returns new loop count
    
    // Timelooper management (single timelooper context)
    pub fn create_new_timelooper(&mut self, name: String, affinity_id: Uuid) -> Result<(), GameError>;
    pub fn load_timelooper(&mut self, timelooper_id: Uuid) -> Result<(), GameError>;
    pub fn save_current_timelooper(&mut self) -> Result<(), GameError>;
    pub fn get_current_timelooper(&self) -> Option<&Timelooper>;
    pub fn get_current_timelooper_mut(&mut self) -> Option<&mut Timelooper>;
    
    // Simplified evaluation methods (no timelooper parameter needed)
    pub fn evaluate_skill_requirement(&self, skill_definition_id: &Uuid, required_level: u32) -> Result<bool, GameError>;
    pub fn can_acquire_ability(&self, ability_definition_id: &Uuid) -> Result<bool, GameError>;
    pub fn can_use_card(&self, card_instance_id: &Uuid, context: &GameContext) -> Result<bool, GameError>;
    
    // Simplified instance management methods (operate on current timelooper, use game time)
    pub fn train_attribute(&mut self, attribute_instance_id: &Uuid, steps: u64) -> Result<AttributeTrainingResult, GameError>;
    pub fn acquire_affinity(&mut self, affinity_definition_id: &Uuid) -> Result<Uuid, GameError>;
    pub fn advance_affinity_mastery(&mut self, affinity_instance_id: &Uuid, levels: u32) -> Result<MasteryResult, GameError>;
    pub fn apply_temporary_modifier(&mut self, attribute_instance_id: &Uuid, modifier: TemporaryModifier) -> Result<(), GameError>;
    pub fn process_expired_modifiers(&mut self) -> Vec<Uuid>; // Returns expired modifier IDs
    
    // Definition access (unchanged)
    pub fn get_available_affinities(&self) -> Vec<&AffinityDefinition>;
    pub fn validate_current_timelooper(&self) -> Result<(), ValidationError>;
}
```

**Quality Gates**:
- 100% test coverage for data-driven operations
- All evaluation methods tested with various scenarios
- Invalid reference handling verified
- Performance tested with large datasets

#### **Step 4.2: Data-Driven Character Creation Engine** ⭐ **MANDATORY**
- Character creation using loaded affinity and skill definitions
- Dynamic validation based on external data constraints
- Real-time feedback using loaded descriptions and requirements

**Creation Flow with External Data**:
1. **Name Input**: Standard validation (independent of external data)
2. **Affinity Selection**: Load all available affinities from game data, show descriptions
3. **Initial Attribute Distribution**: Use loaded attribute definitions for constraints
4. **Starting Benefits**: Apply affinity bonuses and starting items from definitions
5. **Validation**: Ensure all UUIDs reference valid game objects
6. **Character Summary**: Display using loaded names and descriptions
7. **Save**: Store with validated UUID references

**Simplified Character Creation** (Engine Context):
```rust
impl GameEngine {
    // Character creation operates directly on engine's current timelooper
    pub fn start_character_creation(&mut self) -> Result<(), CreationError>;
    pub fn get_available_affinities(&self) -> Vec<&AffinityDefinition>;
    pub fn set_character_name(&mut self, name: String) -> Result<(), CreationError>;
    pub fn select_affinity(&mut self, affinity_id: &Uuid) -> Result<(), CreationError>;
    pub fn get_attribute_constraints(&self, attribute_definition_id: &Uuid) -> Option<(u32, u32)>;
    pub fn finalize_character_creation(&mut self) -> Result<(), CreationError>;
    
    // Instance creation during character creation (operates on engine's timelooper)
    fn create_starting_skills(&mut self, affinity_id: &Uuid) -> Result<Vec<Uuid>, CreationError>;
    fn create_starting_abilities(&mut self, affinity_id: &Uuid) -> Result<Vec<Uuid>, CreationError>;
    fn create_starting_cards(&mut self, affinity_id: &Uuid) -> Result<Vec<Uuid>, CreationError>;
    fn create_starting_items(&mut self, affinity_id: &Uuid) -> Result<Vec<Uuid>, CreationError>;
}
```

**Simplified Character Creation Flow**:
1. **Start Creation**: `engine.start_character_creation()` - initializes empty timelooper in engine
2. **Set Name**: `engine.set_character_name("Hero")` - validates and sets name
3. **Choose Affinity**: `engine.select_affinity(affinity_id)` - applies bonuses, creates starting instances
4. **Finalize**: `engine.finalize_character_creation()` - validates and saves new timelooper
5. **Ready to Play**: Engine now has active timelooper ready for gameplay

**Quality Gates**:
- All creation paths tested with different external data configurations
- Dynamic validation tested with modified game data
- Affinity application tested against loaded definitions
- Character generation reproducible with same data

### **Phase 5: API Layer Development** (Estimated: 3-4 hours)
**Goal**: Create REST endpoints for Timelooper management with full API testing

#### **Step 5.1: Timelooper CRUD Endpoints with Data Validation** ⭐ **MANDATORY**
- RESTful API with game data context for validation
- UUID reference validation against loaded game data
- Rich error responses with game data context

**Game Session Endpoints** (Single Active Timelooper):
```
POST   /api/game/new                       # Create new game with new timelooper
POST   /api/game/load/{save_id}            # Load existing timelooper save
GET    /api/game/current                   # Get current active timelooper
PUT    /api/game/save                      # Save current game state
GET    /api/game/saves                     # List all available saves
DELETE /api/game/saves/{save_id}           # Delete a save file

# Current timelooper instance management (kickoff phase - simplified)
POST   /api/game/affinities                # Add new affinity instance from definition
PUT    /api/game/affinities/{instance_id}  # Modify affinity mastery level
DELETE /api/game/affinities/{instance_id}  # Remove affinity instance

PUT    /api/game/attributes/{instance_id}  # Modify attribute value (training)
GET    /api/game/attributes                # Get all current attribute instances

# 🔮 Future endpoints (not implemented in kickoff):
# POST   /api/game/skills                    # Add new skill instance from definition
# PUT    /api/game/skills/{instance_id}      # Train/modify skill instance  
# DELETE /api/game/skills/{instance_id}      # Remove skill instance
# POST   /api/game/abilities                 # Acquire new ability instance
# POST   /api/game/items                     # Add new item instance

# Evaluation endpoints (kickoff phase - simplified)
GET    /api/game/evaluate/affinity/{definition_id}         # Check if can acquire affinity
GET    /api/game/evaluate/attribute/{definition_id}/{value} # Check if meets attribute requirement

# 🔮 Future evaluation endpoints:
# GET    /api/game/evaluate/skill/{definition_id}/{level}    # Check if meets skill requirement
# GET    /api/game/evaluate/ability/{definition_id}          # Check if can acquire ability
```

**Game Definitions Endpoints** (Kickoff Phase - Simplified):
```
GET    /api/definitions/affinities         # List all available affinity definitions
GET    /api/definitions/attributes         # List all available attribute definitions
GET    /api/definitions/affinity/{id}      # Get specific affinity definition
GET    /api/definitions/attribute/{id}     # Get specific attribute definition
POST   /api/definitions/reload             # Reload definitions from files (dev/admin)
GET    /api/definitions/validate           # Validate definition cross-references

# 🔮 Future definition endpoints:
# GET    /api/definitions/skills             # List all available skill definitions
# GET    /api/definitions/abilities          # List all available ability definitions
# GET    /api/definitions/items              # List all available item definitions
# GET    /api/definitions/cards              # List all available card definitions
```

**Quality Gates**:
- 100% test coverage for all endpoints with game data validation
- Invalid UUID reference handling tested and documented
- Game data reload tested without breaking active sessions
- Performance tested with large game datasets

#### **Step 5.2: Game Session Management** ⭐ **MANDATORY**
- Session creation and lifecycle management
- Save/load game state endpoints
- Session validation and security

**Endpoints**:
```
POST   /api/sessions             # Start new game session
GET    /api/sessions/{id}        # Get session status
PUT    /api/sessions/{id}/save   # Save current game state
DELETE /api/sessions/{id}        # End game session
```

**Quality Gates**:
- Session state consistency tested
- Concurrent session handling verified
- Auto-save functionality implemented

#### **Step 5.3: Health and Status Endpoints** 🌟 **NICE-TO-HAVE**
- System health monitoring
- Storage status and statistics
- Performance metrics endpoint

### **Phase 6: Integration and Main Application** (Estimated: 2-3 hours)
**Goal**: Integrate all components into a working application with proper configuration

#### **Step 6.1: Application Configuration** ⭐ **MANDATORY**
- Environment-based configuration
- Logging setup and configuration
- Server startup and shutdown handling

**Quality Gates**:
- All configuration scenarios tested
- Graceful shutdown implemented
- Logging levels properly configured

#### **Step 6.2: Main Application Assembly with Game Data Loading** ⭐ **MANDATORY**
- Initialize game data loading at startup
- Wire all components with game data context
- Set up routing and middleware with data validation
- Configure CORS and security headers

**Startup Sequence**:
1. **Load Game Data**: Initialize GameDataLoader from external files
2. **Validate Data**: Cross-reference validation of all game objects
3. **Initialize Storage**: Set up timelooper repository
4. **Create Game Engine**: Wire engine with loaded data and storage
5. **Start API Server**: Configure routes with game data context
6. **Health Checks**: Verify all systems operational

**Quality Gates**:
- Integration tests cover full request flows with game data
- Game data loading failure handling tested
- Invalid game data scenarios tested
- Hot reload of game data tested (for future game editor)
- Performance under load with large datasets validated

#### **Step 6.3: Sample Game Data Creation** ⭐ **MANDATORY**
- Create initial game data files for testing and demonstration
- Implement basic affinities, skills, and attributes
- Validate complete data loading and character creation flow

**Sample Definition Data Required** (Kickoff Phase):
```json
// game_data/core/affinities.json  
{
  "affinities": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "name": "Hand-to-Hand Combat",
      "description": "Masters of physical combat and martial prowess",
      "lore_text": "Ancient warriors who perfected the art of unarmed combat through discipline and training.",
      "attribute_bonuses": {
        "550e8400-e29b-41d4-a716-446655440100": 5,  // Physical attribute +5
        "550e8400-e29b-41d4-a716-446655440101": 2   // Mental attribute +2
      },
      "mastery_benefits": {
        "25": "Enhanced physical damage resistance",
        "50": "Ability to disarm opponents", 
        "75": "Master-level combat techniques",
        "100": "Legendary warrior status"
      },
      "cultural_background": "Monasteries and warrior academies"
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440002",
      "name": "Psionic",
      "description": "Wielders of mental and psychic abilities",
      "lore_text": "Rare individuals born with or trained in the manipulation of psychic energies.",
      "attribute_bonuses": {
        "550e8400-e29b-41d4-a716-446655440101": 7,  // Mental attribute +7
        "550e8400-e29b-41d4-a716-446655440100": 1   // Physical attribute +1
      },
      "mastery_benefits": {
        "25": "Basic telepathic communication",
        "50": "Telekinetic manipulation of objects",
        "75": "Mind reading and influence",
        "100": "Reality manipulation"
      },
      "cultural_background": "Secret societies and research institutes"
    }
  ]
}

// game_data/core/attributes.json
{
  "attributes": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440100", 
      "name": "Physical",
      "description": "Represents bodily strength, endurance, and physical capabilities",
      "min_value": 1,
      "max_value": 100,
      "default_value": 10,
      "training_difficulty": 1.0
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440101",
      "name": "Mental", 
      "description": "Represents intelligence, willpower, and mental acuity",
      "min_value": 1,
      "max_value": 100,
      "default_value": 10,
      "training_difficulty": 1.2
    }
  ]
}

```

**Sample Instance Creation** (Runtime Generated with Effect System):
```rust
// When creating a timelooper with Hand-to-Hand affinity:
let current_time = GameTimeStep::zero(); // Character creation happens at step 0
let physical_attr_id = "550e8400-e29b-41d4-a716-446655440100".parse().unwrap();
let mental_attr_id = "550e8400-e29b-41d4-a716-446655440101".parse().unwrap();
let affinity_id = "550e8400-e29b-41d4-a716-446655440001".parse().unwrap();

// Create affinity instance
let affinity_instance = AffinityInstance {
    id: Uuid::new_v4(),
    definition_id: affinity_id,
    acquired_at: current_time.clone(),
    base_mastery_level: 0,                    // Base mastery without effects
    effects: vec![],                          // 🎯 No effects initially
    mastery_history: vec![],
};

// Create physical attribute with base value
let mut physical_attribute = AttributeInstance {
    id: Uuid::new_v4(),
    definition_id: physical_attr_id,
    base_value: 10,                           // ✅ Base value only (definition default)
    effects: vec![],                          // 🎯 Effects will be added
    training_history: vec![],
};

// Add affinity bonus as permanent effect
let physical_bonus = Effect::affinity_attribute_bonus(
    affinity_id,
    physical_attr_id,
    5,                                        // +5 bonus from Hand-to-Hand affinity
    current_time.clone(),
);
physical_attribute.add_effect(physical_bonus).unwrap();

// Create mental attribute with base value
let mut mental_attribute = AttributeInstance {
    id: Uuid::new_v4(),
    definition_id: mental_attr_id,
    base_value: 10,                           // ✅ Base value only
    effects: vec![],                          // 🎯 Effects will be added
    training_history: vec![],
};

// Add affinity bonus as permanent effect
let mental_bonus = Effect::affinity_attribute_bonus(
    affinity_id,
    mental_attr_id,
    2,                                        // +2 bonus from Hand-to-Hand affinity
    current_time.clone(),
);
mental_attribute.add_effect(mental_bonus).unwrap();

let new_timelooper = Timelooper {
    id: Uuid::new_v4(),
    name: "Hero".to_string(),
    affinities: vec![affinity_instance],
    attributes: HashMap::from([
        (physical_attribute.instance_id, physical_attribute),
        (mental_attribute.instance_id, mental_attribute),
    ]),
    base_money: 100,                          // Base money without effects
    effects: vec![],                          // 🎯 Global timelooper effects
    current_game_time: GameTimeStep::zero(),
    loop_count: 0,
    created_at: Utc::now(),
    last_saved_at: Utc::now(),
};

// Effective values calculated dynamically:
// physical_attribute.current_value(&current_time) == 15 (10 base + 5 effect)
// mental_attribute.current_value(&current_time) == 12 (10 base + 2 effect)  
// new_timelooper.current_money(&current_time) == 100 (no money effects yet)

// 🔮 Future instance types will be added in later phases:
// - SkillInstance, AbilityInstance, ItemInstance, CardInstance, etc.
```

**Quality Gates**:
- Complete character creation possible with simplified timelooper structure
- Affinity and attribute instances created and managed properly
- All cross-references validated between instances and definitions
- API endpoints functional with simplified data model
- Character save/load tested with kickoff phase objects only
- UUID lifecycle preserved for affinities and attributes

#### **Step 6.4: CLI Interface (Temporary)** 🌟 **NICE-TO-HAVE**
- Simple command-line interface for testing
- Interactive character creation via CLI using loaded game data
- Game data inspection and validation tools

## 📊 **Quality Standards Enforcement**

### **Mandatory Quality Gates for Each Step**
1. **Build Success**: `cargo build` must complete with 0 warnings
2. **Clippy Compliance**: `cargo clippy` must show 0 suggestions
3. **Test Coverage**: `cargo test` must achieve 100% coverage for new code
4. **Strict Quality**: `cargo clippy -- -D warnings` must pass
5. **Documentation**: All public APIs must have doc comments
6. **Integration**: All components must work together seamlessly

### **Quality Validation Workflow**
```bash
# Before completing any step:
cargo build                      # Must show: 0 warnings
cargo clippy                     # Must show: 0 suggestions  
cargo test                       # Must show: All tests pass
cargo clippy -- -D warnings      # Must pass strict mode
cargo doc --no-deps             # Must generate clean docs
```

### **Testing Requirements**
- **Unit Tests**: Every function and method
- **Integration Tests**: Cross-component functionality  
- **Error Path Tests**: All error conditions
- **Edge Case Tests**: Boundary values and invalid inputs
- **Performance Tests**: Basic benchmarks for critical paths

### **UUID Lifecycle Testing Requirements** 🧪 **CRITICAL**
- **Definition UUID Stability**: Same definition UUIDs across multiple game sessions
- **Instance UUID Generation**: New instances get unique UUIDs, never collide
- **Save/Load UUID Preservation**: Round-trip testing (save -> load -> verify exact UUIDs)
- **No Auto-Generation**: Compilation fails if Default generates UUIDs
- **Cross-Reference Validation**: All instance definition_ids reference valid loaded definitions
- **UUID Uniqueness**: No duplicate instance UUIDs within same timelooper

### **Game Time System Testing Requirements** ⏰ **CRITICAL**
- **Game Time Independence**: Game mechanics work identically regardless of real-world time
- **Save/Load Game Time Preservation**: Game time preserved exactly across save/load cycles
- **Time Step Progression**: Advancing time updates all relevant timestamps correctly
- **Loop Reset Functionality**: Time resets to zero when starting new timeloop
- **Expiration Processing**: Temporary modifiers expire at correct game time steps
- **Time Ordering**: Game events maintain correct chronological order by game time
- **Real vs Game Time Separation**: No game mechanics use real-world time accidentally

### **Generic Effect System Testing Requirements** 🎯 **CRITICAL**
- **Effect Application**: Effects correctly modify target values according to their modification type
- **Effect Stacking**: Multiple effects combine correctly according to stacking rules
- **Effect Expiration**: Effects expire at correct game time and are automatically removed
- **Effect Conditions**: Conditional effects activate/deactivate based on game state
- **Effect Priority**: Effects apply in correct order based on priority values
- **Save/Load Effect Preservation**: All effect properties preserved across save/load cycles
- **Cross-Instance Effects**: Effects can target any instance type consistently
- **Effect UUID Lifecycle**: Effect UUIDs follow same lifecycle rules as other objects

```rust
#[cfg(test)]
mod uuid_lifecycle_tests {
    #[test]
    fn definition_uuids_stable_across_sessions() {
        // Load definitions twice, verify same UUIDs
    }
    
    #[test]
    fn instance_uuids_preserved_in_save_load() {
        // Create instances -> save -> load -> verify same UUIDs
    }
    
    #[test]
    fn new_instances_get_unique_uuids() {
        // Create multiple instances, verify all UUIDs different
    }
    
    #[test]
    fn no_auto_uuid_generation_in_constructors() {
        // Ensure compilation fails for Default with auto UUIDs
    }
}

#[cfg(test)]
mod game_time_tests {
    #[test]
    fn game_time_advances_correctly() {
        // Verify time step progression works as expected
    }
    
    #[test]
    fn game_time_preserved_in_save_load() {
        // Create game -> advance time -> save -> load -> verify same time
    }
    
    #[test]
    fn loop_reset_clears_game_time() {
        // Advance time -> start new loop -> verify time back to zero
    }
    
    #[test]
    fn effect_expiration_by_game_time() {
        // Create effect with expiration -> advance past expiration -> verify removed
    }
    
    #[test]
    fn real_time_vs_game_time_separation() {
        // Verify no game mechanics accidentally use DateTime<Utc>
    }
    
    #[test]
    fn game_time_ordering_maintained() {
        // Create events in order -> verify chronological ordering by game time
    }
}

#[cfg(test)]
mod effect_system_tests {
    #[test]
    fn additive_effects_stack_correctly() {
        // Multiple additive effects should sum together
    }
    
    #[test]
    fn multiplicative_effects_multiply_correctly() {
        // Multiplicative effects should compound properly
    }
    
    #[test]
    fn effect_stacking_rules_enforced() {
        // Test Replace, Stack, StackHighest, StackNewest behaviors
    }
    
    #[test]
    fn effect_priority_ordering() {
        // Higher priority effects should apply after lower priority
    }
    
    #[test]
    fn conditional_effects_activate_correctly() {
        // Effects with conditions should only apply when conditions met
    }
    
    #[test]
    fn effects_preserved_in_save_load() {
        // Create effects -> save -> load -> verify identical effects
    }
    
    #[test]
    fn effect_expiration_processed_correctly() {
        // Effects should expire and be removed at correct game time
    }
    
    #[test]
    fn cross_instance_effect_targeting() {
        // Effects should work on attributes, affinities, timelooper consistently
    }
    
    #[test]
    fn effect_uuid_lifecycle_management() {
        // Effect UUIDs should follow same rules as other objects
    }
}
```

## 📅 **Development Timeline**

### **Week 1: Foundation (Phases 1-2)**
- Day 1-2: Project setup and dependencies
- Day 3-5: Timelooper model and validation
- Weekend: Testing and quality validation

### **Week 2: Core Systems (Phases 3-4)**
- Day 1-3: Storage layer implementation
- Day 4-5: Game engine foundation
- Weekend: Integration and testing

### **Week 3: API and Integration (Phases 5-6)**
- Day 1-3: REST API development
- Day 4-5: Main application assembly
- Weekend: End-to-end testing and documentation

## 🎯 **Success Criteria**

### **Minimum Viable Product Delivery**
- ✅ Fully functional Timelooper creation system
- ✅ Persistent JSON-based save system
- ✅ RESTful API for all operations
- ✅ 100% test coverage across all components
- ✅ Zero warnings and clippy issues
- ✅ Complete documentation

### **Technical Excellence**
- ✅ Follows TECHDESIGN.md architecture principles
- ✅ Implements CODE_QUALITY_STANDARDS.md requirements
- ✅ Uses only Rust-based services (SERVER_GUIDELINES.md)
- ✅ Scalable foundation for future features
- ✅ Production-ready error handling

### **Future Readiness**
- ✅ Architecture supports Bevy frontend integration
- ✅ Game engine ready for rule system expansion
- ✅ Storage layer supports additional entity types
- ✅ API design supports real-time features (future WebSocket)

## �️ **Game Object Editor Preparation**

### **Editor-Ready Architecture Benefits**
The data-driven architecture implemented in this kickoff plan directly enables the future game object editor:

1. **Standardized JSON Schema** - All game objects follow consistent UUID-based structure
2. **Hot Reload Capability** - Game engine can reload data without restart  
3. **Cross-Reference Validation** - Editor can validate UUID references in real-time
4. **API Foundation** - CRUD endpoints ready for editor integration
5. **Type Safety** - Rust definitions ensure editor generates valid data structures

### **Editor Foundation Elements (Built into Kickoff)**
- **Game Data API Endpoints** - Full CRUD for all game object types
- **Reference Validation System** - Ensures UUID integrity across all objects
- **Schema Export Capability** - Can generate JSON schemas for editor forms
- **Real-time Validation** - Live feedback on game data modifications
- **Atomic Updates** - Safe modification of game data with rollback capability

### **Post-Kickoff Editor Development Path**
1. **Web Editor Frontend** - React/Vue interface for game object creation
2. **Schema Generation** - Auto-generate forms from Rust type definitions  
3. **Visual Relationship Mapping** - Show UUID dependencies between objects
4. **Content Testing** - Test game objects in isolated sandbox environment
5. **Version Control Integration** - Track changes to game data definitions

## �📋 **Next Steps After Kickoff**

### **Immediate Follow-up** (Phase 7+)
1. **Game Object Editor** - Web-based editor for creating/modifying affinities, skills, abilities, cards, etc.
2. **Enhanced Game Data Validation** - Schema validation, dependency checking, circular reference detection
3. **Bevy Frontend Integration** - Connect game client to REST API with game data context
4. **Advanced Card System** - Complete card mechanics with external definitions
5. **Event System Foundation** - External event definitions and triggers

### **Long-term Roadmap**
1. **Advanced Game Features** - Skills, abilities, status system
2. **World Simulation** - NPCs, events, and world state progression
3. **AI and Procedural Generation** - Dynamic content creation
4. **Performance Optimization** - Database integration, caching
5. **Production Deployment** - Docker, monitoring, CI/CD

## 🏆 **Architectural Decision: Single Timelooper Context**

### **Final Assessment** ✅

**RECOMMENDATION: Adopt Single Timelooper Architecture**

The single timelooper architecture is the correct choice for this game because:

1. **Game Design Alignment**: Perfect match for timeloop concept (one character, multiple loops)
2. **Simplified Codebase**: Eliminates repetitive timelooper parameters across all methods
3. **Cleaner APIs**: More intuitive endpoints (`/api/game/skills` vs `/api/timeloopers/{id}/skills`)
4. **Better Testing**: Easier test setup without timelooper parameter management
5. **Future-Proof**: Can easily extend for world state, NPCs, events without timelooper complexity

### **No Significant Arguments Against** ✅

**Potential Concerns Analyzed**:
- ❓ **Multiplayer Support**: Not needed - game is single-player timeloop experience
- ❓ **Testing Flexibility**: Solved with `set_timelooper_for_testing()` method
- ❓ **Save/Load Complexity**: Actually simplified - direct engine state switching
- ❓ **Code Reusability**: Engine methods become more focused and specialized

**Conclusion**: The single timelooper architecture provides significant benefits with no meaningful drawbacks for this specific game design.

---

## 📚 **References and Dependencies**

- **GAMEDESIGN.md**: Core game mechanics and requirements
- **TECHDESIGN.md**: Architecture patterns and folder structure  
- **GAMECLIENT.md**: Future frontend architecture planning
- **CODE_QUALITY_STANDARDS.md**: Quality enforcement requirements
- **SERVER_GUIDELINES.md**: Rust-only server policy

This kickoff plan provides a solid foundation for the Timeloop game while maintaining the highest code quality standards and preparing for future expansion into a full-featured gaming experience.