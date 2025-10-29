// Persistence traits - business logic depends on THESE, not implementations
// CRITICAL: Business logic has ZERO knowledge of storage implementation

use crate::models::definitions::AttributeDefinition;
use crate::models::common::AttributeId;
use thiserror::Error;

/// Persistence error - no details about storage type
#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("Item not found")]
    NotFound,
    
    #[error("Storage operation failed: {0}")]
    StorageError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("IO error: {0}")]
    IoError(String),
}

// Convert from std::io::Error
impl From<std::io::Error> for PersistenceError {
    fn from(err: std::io::Error) -> Self {
        PersistenceError::IoError(err.to_string())
    }
}

// Convert from serde_json::Error
impl From<serde_json::Error> for PersistenceError {
    fn from(err: serde_json::Error) -> Self {
        PersistenceError::SerializationError(err.to_string())
    }
}

// Convert to BusinessError
impl From<PersistenceError> for crate::business::BusinessError {
    fn from(err: PersistenceError) -> Self {
        match err {
            PersistenceError::NotFound => crate::business::BusinessError::NotFound,
            PersistenceError::StorageError(msg) => crate::business::BusinessError::PersistenceError(msg),
            PersistenceError::SerializationError(msg) => crate::business::BusinessError::PersistenceError(msg),
            PersistenceError::IoError(msg) => crate::business::BusinessError::PersistenceError(msg),
        }
    }
}

/// Persistence trait for attributes - business logic depends on THIS, not implementation
/// Business logic doesn't know if this writes to file, database, memory, or network
#[async_trait::async_trait]
pub trait AttributePersistence: Send + Sync {
    /// Save or update an attribute
    /// Business logic doesn't know if this writes to file, database, or memory
    async fn save(&self, attr: &AttributeDefinition) -> Result<(), PersistenceError>;
    
    /// Delete an attribute by ID
    async fn delete(&self, id: &AttributeId) -> Result<(), PersistenceError>;
    
    /// Get a single attribute by ID
    async fn get(&self, id: &AttributeId) -> Result<Option<AttributeDefinition>, PersistenceError>;
    
    /// List all attributes
    async fn list_all(&self) -> Result<Vec<AttributeDefinition>, PersistenceError>;
    
    /// Check if an attribute with this name exists
    async fn exists_by_name(&self, name: &str) -> Result<bool, PersistenceError>;
    
    /// Search attributes by name (contains match, case-insensitive)
    async fn search_by_name(&self, query: &str) -> Result<Vec<AttributeDefinition>, PersistenceError>;
}

/// Persistence trait for cards - business logic depends on THIS, not implementation
#[async_trait::async_trait]
pub trait CardPersistence: Send + Sync {
    /// Save or update a card
    async fn save(&self, card: &crate::models::cards::CardDefinition) -> Result<(), PersistenceError>;
    
    /// Delete a card by ID
    async fn delete(&self, id: &crate::models::cards::CardId) -> Result<(), PersistenceError>;
    
    /// Get a single card by ID
    async fn get(&self, id: &crate::models::cards::CardId) -> Result<Option<crate::models::cards::CardDefinition>, PersistenceError>;
    
    /// List all cards
    async fn list_all(&self) -> Result<Vec<crate::models::cards::CardDefinition>, PersistenceError>;
    
    /// Check if a card with this caption exists
    async fn exists_by_caption(&self, caption: &str) -> Result<bool, PersistenceError>;
    
    /// Search cards by caption (contains match, case-insensitive)
    async fn search_by_caption(&self, query: &str) -> Result<Vec<crate::models::cards::CardDefinition>, PersistenceError>;
}

// Future traits for other definition types
// pub trait AffinityPersistence: Send + Sync { ... }
// pub trait EffectPersistence: Send + Sync { ... }
