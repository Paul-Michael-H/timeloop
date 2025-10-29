// Attribute validation logic
// CRITICAL: Validation is independent and injectable

use super::ValidationError;
use crate::models::definitions::AttributeDefinition;

/// Trait for attribute validation (injectable)
pub trait AttributeValidator: Send + Sync {
    /// Validate an attribute definition
    /// `all` contains other attributes for uniqueness checks (excluding the one being validated for updates)
    fn validate(&self, attr: &AttributeDefinition, all: &[AttributeDefinition]) 
        -> Result<(), ValidationError>;
}

/// Production implementation
pub struct AttributeValidatorImpl;

impl AttributeValidatorImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AttributeValidatorImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl AttributeValidator for AttributeValidatorImpl {
    fn validate(&self, attr: &AttributeDefinition, _all: &[AttributeDefinition]) 
        -> Result<(), ValidationError> {
        // 1. Name validation
        if attr.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        // NOTE: Duplicate name checking is handled by business logic layer
        // using persistence.exists_by_name(), not here in validation
        
        // 2. Description validation
        if attr.description.trim().is_empty() {
            return Err(ValidationError::EmptyDescription);
        }
        
        // 3. Value range validation
        if attr.min_value > attr.max_value {
            return Err(ValidationError::InvalidRange {
                min: attr.min_value,
                max: attr.max_value,
            });
        }
        
        // 4. Base value must be in range
        if attr.base_value < attr.min_value || attr.base_value > attr.max_value {
            return Err(ValidationError::BaseValueOutOfRange {
                base: attr.base_value,
                min: attr.min_value,
                max: attr.max_value,
            });
        }
        
        // 5. Training difficulty must be at least 1%
        if attr.training_difficulty.get() == 0 {
            return Err(ValidationError::InvalidTrainingDifficulty);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::common::{AttributeId, Percentage};
    use crate::models::definitions::AttributeCategory;
    
    fn create_valid_attribute() -> AttributeDefinition {
        AttributeDefinition {
            id: AttributeId::new(),
            name: "Test Attribute".to_string(),
            description: "Test description".to_string(),
            category: AttributeCategory::Physical,
            base_value: 10,
            min_value: 1,
            max_value: 100,
            training_difficulty: Percentage::new(100),
            icon: None,
        }
    }
    
    #[test]
    fn test_valid_attribute_passes() {
        let validator = AttributeValidatorImpl::new();
        let attr = create_valid_attribute();
        
        assert!(validator.validate(&attr, &[]).is_ok());
    }
    
    #[test]
    fn test_empty_name_fails() {
        let validator = AttributeValidatorImpl::new();
        let mut attr = create_valid_attribute();
        attr.name = "".to_string();
        
        let result = validator.validate(&attr, &[]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::EmptyName));
    }
    
    #[test]
    fn test_whitespace_name_fails() {
        let validator = AttributeValidatorImpl::new();
        let mut attr = create_valid_attribute();
        attr.name = "   ".to_string();
        
        let result = validator.validate(&attr, &[]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_duplicate_name_fails() {
        // NOTE: Duplicate name checking moved to business logic layer
        // This test is no longer relevant as validator doesn't check duplicates
        // The business logic uses persistence.exists_by_name() instead
        // See business_logic_tests.rs for duplicate name tests
    }
    
    #[test]
    fn test_duplicate_name_case_insensitive() {
        // NOTE: Duplicate name checking moved to business logic layer
        // This test is no longer relevant as validator doesn't check duplicates
        // See business_logic_tests.rs for duplicate name tests
    }
    
    #[test]
    fn test_empty_description_fails() {
        let validator = AttributeValidatorImpl::new();
        let mut attr = create_valid_attribute();
        attr.description = "".to_string();
        
        let result = validator.validate(&attr, &[]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::EmptyDescription));
    }
    
    #[test]
    fn test_invalid_range_fails() {
        let validator = AttributeValidatorImpl::new();
        let mut attr = create_valid_attribute();
        attr.min_value = 100;
        attr.max_value = 50;
        
        let result = validator.validate(&attr, &[]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::InvalidRange { .. }));
    }
    
    #[test]
    fn test_base_below_min_fails() {
        let validator = AttributeValidatorImpl::new();
        let mut attr = create_valid_attribute();
        attr.min_value = 10;
        attr.base_value = 5;
        attr.max_value = 100;
        
        let result = validator.validate(&attr, &[]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::BaseValueOutOfRange { .. }));
    }
    
    #[test]
    fn test_base_above_max_fails() {
        let validator = AttributeValidatorImpl::new();
        let mut attr = create_valid_attribute();
        attr.min_value = 1;
        attr.base_value = 150;
        attr.max_value = 100;
        
        let result = validator.validate(&attr, &[]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_zero_training_difficulty_fails() {
        let validator = AttributeValidatorImpl::new();
        let mut attr = create_valid_attribute();
        attr.training_difficulty = Percentage::new(0);
        
        let result = validator.validate(&attr, &[]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::InvalidTrainingDifficulty));
    }
}
