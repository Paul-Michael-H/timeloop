// Business logic layer - contains all game logic and rules
// This layer is dependency-injected and testable
// NO direct file system or HTTP access - all through traits

pub mod definitions;
pub mod validation;
pub mod game_state;

// Re-export main traits for convenience
pub use definitions::{AttributeService, AffinityService, EffectService};
pub use validation::{AttributeValidator, ValidationError};

// Common business error type
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BusinessError {
    #[error("Item not found")]
    NotFound,
    
    #[error("Duplicate name: {0}")]
    DuplicateName(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("ID mismatch: expected {expected}, got {actual}")]
    IdMismatch { expected: String, actual: String },
    
    #[error("Persistence error: {0}")]
    PersistenceError(String),
    
    #[error("Business rule violation: {0}")]
    BusinessRuleViolation(String),
}
