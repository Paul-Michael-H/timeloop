// Card definitions for encounter system

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a card
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardId(Uuid);

impl CardId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

}

impl Default for CardId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CardId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of card
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CardType {
    #[default]
    Action,
    Equipment,
}

impl CardType {
    pub fn all() -> Vec<CardType> {
        vec![CardType::Action, CardType::Equipment]
    }

    pub fn as_str(&self) -> &str {
        match self {
            CardType::Action => "Action",
            CardType::Equipment => "Equipment",
        }
    }
}

impl std::fmt::Display for CardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Defense type for cards
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DefenseType {
    #[default]
    Dodge,
    Armor,
    Shield,
    Barrier,
    Reflect,
    Absorb,
    Nullify,
}

impl DefenseType {
    pub fn all() -> Vec<DefenseType> {
        vec![
            DefenseType::Dodge,
            DefenseType::Armor,
            DefenseType::Shield,
            DefenseType::Barrier,
            DefenseType::Reflect,
            DefenseType::Absorb,
            DefenseType::Nullify,
        ]
    }

    pub fn as_str(&self) -> &str {
        match self {
            DefenseType::Dodge => "Dodge",
            DefenseType::Armor => "Armor",
            DefenseType::Shield => "Shield",
            DefenseType::Barrier => "Barrier",
            DefenseType::Reflect => "Reflect",
            DefenseType::Absorb => "Absorb",
            DefenseType::Nullify => "Nullify",
        }
    }
}

impl std::fmt::Display for DefenseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Defense strength levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DefenseStrength {
    #[default]
    Light,
    Medium,
    Severe,
    Deadly,
}

impl DefenseStrength {
    pub fn all() -> Vec<DefenseStrength> {
        vec![
            DefenseStrength::Light,
            DefenseStrength::Medium,
            DefenseStrength::Severe,
            DefenseStrength::Deadly,
        ]
    }

    pub fn as_str(&self) -> &str {
        match self {
            DefenseStrength::Light => "Light",
            DefenseStrength::Medium => "Medium",
            DefenseStrength::Severe => "Severe",
            DefenseStrength::Deadly => "Deadly",
        }
    }
}

impl std::fmt::Display for DefenseStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Damage types that cards can be effective/ineffective against
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DamageType {
    #[default]
    Slash,
    Pierce,
    Bludgeon,
    Electrical,
    Fire,
    Cold,
    Quantum,
    Nano,
    Bio,
    Acid,
    Poison,
    Rad,
    Energy,
    Gravity,
    Kinetic,
    Blast,
    Implosion,
    Dimensional,
    EM,
}

impl DamageType {
    pub fn all() -> Vec<DamageType> {
        vec![
            DamageType::Slash,
            DamageType::Pierce,
            DamageType::Bludgeon,
            DamageType::Electrical,
            DamageType::Fire,
            DamageType::Cold,
            DamageType::Quantum,
            DamageType::Nano,
            DamageType::Bio,
            DamageType::Acid,
            DamageType::Poison,
            DamageType::Rad,
            DamageType::Energy,
            DamageType::Gravity,
            DamageType::Kinetic,
            DamageType::Blast,
            DamageType::Implosion,
            DamageType::Dimensional,
            DamageType::EM,
        ]
    }

    pub fn as_str(&self) -> &str {
        match self {
            DamageType::Slash => "Slash",
            DamageType::Pierce => "Pierce",
            DamageType::Bludgeon => "Bludgeon",
            DamageType::Electrical => "Electrical",
            DamageType::Fire => "Fire",
            DamageType::Cold => "Cold",
            DamageType::Quantum => "Quantum",
            DamageType::Nano => "Nano",
            DamageType::Bio => "Bio",
            DamageType::Acid => "Acid",
            DamageType::Poison => "Poison",
            DamageType::Rad => "Rad",
            DamageType::Energy => "Energy",
            DamageType::Gravity => "Gravity",
            DamageType::Kinetic => "Kinetic",
            DamageType::Blast => "Blast",
            DamageType::Implosion => "Implosion",
            DamageType::Dimensional => "Dimensional",
            DamageType::EM => "EM",
        }
    }
}

impl std::fmt::Display for DamageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Equipment slot for cards
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EquipmentSlot {
    Head,
    Body,
    #[default]
    Other,
}

impl EquipmentSlot {
    pub fn all() -> Vec<EquipmentSlot> {
        vec![
            EquipmentSlot::Head,
            EquipmentSlot::Body,
            EquipmentSlot::Other,
        ]
    }

    pub fn as_str(&self) -> &str {
        match self {
            EquipmentSlot::Head => "Head",
            EquipmentSlot::Body => "Body",
            EquipmentSlot::Other => "Other",
        }
    }
}

impl std::fmt::Display for EquipmentSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Rarity levels for cards
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Unique,
}

impl Rarity {
    pub fn all() -> Vec<Rarity> {
        vec![
            Rarity::Common,
            Rarity::Uncommon,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
            Rarity::Unique,
        ]
    }

    pub fn as_str(&self) -> &str {
        match self {
            Rarity::Common => "Common",
            Rarity::Uncommon => "Uncommon",
            Rarity::Rare => "Rare",
            Rarity::Epic => "Epic",
            Rarity::Legendary => "Legendary",
            Rarity::Unique => "Unique",
        }
    }

    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            Rarity::Common => (128, 128, 128),    // Gray
            Rarity::Uncommon => (0, 255, 0),      // Green
            Rarity::Rare => (0, 112, 221),        // Blue
            Rarity::Epic => (163, 53, 238),       // Purple
            Rarity::Legendary => (255, 128, 0),   // Orange
            Rarity::Unique => (255, 215, 0),      // Gold
        }
    }
}

impl std::fmt::Display for Rarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Card definition for encounters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDefinition {
    pub id: CardId,
    pub caption: String,
    pub description: String,
    pub card_type: CardType,
    pub defense_type: Option<DefenseType>,
    pub defense_strength: Option<DefenseStrength>,
    pub effective_against: Vec<DamageType>,
    pub ineffective_against: Vec<DamageType>,
    pub number_of_uses: Option<u32>,  // None = infinite uses
    pub equipment_slot: Option<EquipmentSlot>,
    pub rarity: Rarity,
}

impl CardDefinition {
    pub fn new(caption: String) -> Self {
        Self {
            id: CardId::new(),
            caption,
            description: String::new(),
            card_type: CardType::default(),
            defense_type: None,
            defense_strength: None,
            effective_against: Vec::new(),
            ineffective_against: Vec::new(),
            number_of_uses: None,
            equipment_slot: None,
            rarity: Rarity::default(),
        }
    }

    /// Validate that effective_against and ineffective_against don't overlap
    pub fn validate_damage_types(&self) -> Result<(), String> {
        for damage_type in &self.effective_against {
            if self.ineffective_against.contains(damage_type) {
                return Err(format!(
                    "Damage type {:?} cannot be both effective and ineffective",
                    damage_type
                ));
            }
        }
        Ok(())
    }
}

impl Default for CardDefinition {
    fn default() -> Self {
        Self::new("New Card".to_string())
    }
}
