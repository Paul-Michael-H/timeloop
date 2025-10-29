// Card validation logic

use crate::models::cards::CardDefinition;

/// Validation error types
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    EmptyField(String),
    TooLong(String, usize),
    InvalidValue(String, String),
    ConflictingValues(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyField(field) => write!(f, "Field '{}' cannot be empty", field),
            ValidationError::TooLong(field, max) => write!(f, "Field '{}' exceeds maximum length of {}", field, max),
            ValidationError::InvalidValue(field, reason) => write!(f, "Invalid value for '{}': {}", field, reason),
            ValidationError::ConflictingValues(msg) => write!(f, "Conflicting values: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Trait for card validation
#[async_trait::async_trait]
pub trait CardValidator: Send + Sync {
    fn validate(&self, card: &CardDefinition) -> Result<(), ValidationError>;
}

/// Implementation of card validator
pub struct CardValidatorImpl;

impl CardValidatorImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CardValidatorImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CardValidator for CardValidatorImpl {
    fn validate(&self, card: &CardDefinition) -> Result<(), ValidationError> {
        // Caption must not be empty
        if card.caption.trim().is_empty() {
            return Err(ValidationError::EmptyField("caption".to_string()));
        }
        
        // Caption length limit
        if card.caption.len() > 100 {
            return Err(ValidationError::TooLong("caption".to_string(), 100));
        }
        
        // Description must not be empty
        if card.description.trim().is_empty() {
            return Err(ValidationError::EmptyField("description".to_string()));
        }
        
        // Description length limit
        if card.description.len() > 500 {
            return Err(ValidationError::TooLong("description".to_string(), 500));
        }
        
        // Validate damage type overlap
        card.validate_damage_types()
            .map_err(ValidationError::ConflictingValues)?;
        
        // Number of uses validation
        if let Some(uses) = card.number_of_uses {
            if uses == 0 {
                return Err(ValidationError::InvalidValue(
                    "number_of_uses".to_string(),
                    "must be greater than 0 or None for infinite uses".to_string()
                ));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::cards::{CardDefinition, DamageType};

    fn create_valid_card() -> CardDefinition {
        let mut card = CardDefinition::new("Test Card".to_string());
        card.description = "A test card for validation".to_string();
        card
    }

    #[test]
    fn test_valid_card_passes() {
        let validator = CardValidatorImpl::new();
        let card = create_valid_card();
        
        assert!(validator.validate(&card).is_ok());
    }

    #[test]
    fn test_empty_caption_fails() {
        let validator = CardValidatorImpl::new();
        let mut card = create_valid_card();
        card.caption = "".to_string();
        
        let result = validator.validate(&card);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::EmptyField(_)));
    }

    #[test]
    fn test_whitespace_caption_fails() {
        let validator = CardValidatorImpl::new();
        let mut card = create_valid_card();
        card.caption = "   ".to_string();
        
        let result = validator.validate(&card);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_description_fails() {
        let validator = CardValidatorImpl::new();
        let mut card = create_valid_card();
        card.description = "".to_string();
        
        let result = validator.validate(&card);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::EmptyField(_)));
    }

    #[test]
    fn test_caption_too_long_fails() {
        let validator = CardValidatorImpl::new();
        let mut card = create_valid_card();
        card.caption = "a".repeat(101);
        
        let result = validator.validate(&card);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::TooLong(_, 100)));
    }

    #[test]
    fn test_description_too_long_fails() {
        let validator = CardValidatorImpl::new();
        let mut card = create_valid_card();
        card.description = "a".repeat(501);
        
        let result = validator.validate(&card);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::TooLong(_, 500)));
    }

    #[test]
    fn test_zero_uses_fails() {
        let validator = CardValidatorImpl::new();
        let mut card = create_valid_card();
        card.number_of_uses = Some(0);
        
        let result = validator.validate(&card);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::InvalidValue(_, _)));
    }

    #[test]
    fn test_none_uses_passes() {
        let validator = CardValidatorImpl::new();
        let mut card = create_valid_card();
        card.number_of_uses = None;  // Infinite uses
        
        assert!(validator.validate(&card).is_ok());
    }

    #[test]
    fn test_overlapping_damage_types_fails() {
        let validator = CardValidatorImpl::new();
        let mut card = create_valid_card();
        card.effective_against = vec![DamageType::Fire, DamageType::Cold];
        card.ineffective_against = vec![DamageType::Fire];  // Overlap!
        
        let result = validator.validate(&card);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::ConflictingValues(_)));
    }

    #[test]
    fn test_non_overlapping_damage_types_passes() {
        let validator = CardValidatorImpl::new();
        let mut card = create_valid_card();
        card.effective_against = vec![DamageType::Fire, DamageType::Cold];
        card.ineffective_against = vec![DamageType::Slash, DamageType::Pierce];
        
        assert!(validator.validate(&card).is_ok());
    }
}
