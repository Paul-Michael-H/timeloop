// Validation logic for game definitions
// All validators are trait-based for dependency injection

pub mod attributes;

// Re-export
pub use attributes::{AttributeValidator, AttributeValidatorImpl};

use thiserror::Error;

/// Validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Name cannot be empty")]
    EmptyName,
    
    #[error("Name already exists: {0}")]
    DuplicateName(String),
    
    #[error("Description cannot be empty")]
    EmptyDescription,
    
    #[error("Invalid value range: min ({min}) must be less than or equal to max ({max})")]
    InvalidRange { min: u32, max: u32 },
    
    #[error("Base value ({base}) must be between min ({min}) and max ({max})")]
    BaseValueOutOfRange { base: u32, min: u32, max: u32 },
    
    #[error("Training difficulty must be at least 1%")]
    InvalidTrainingDifficulty,
    
    #[error("Validation failed: {0}")]
    CustomError(String),
}

// Convert ValidationError to BusinessError
impl From<ValidationError> for crate::business::BusinessError {
    fn from(err: ValidationError) -> Self {
        crate::business::BusinessError::ValidationError(err.to_string())
    }
}
