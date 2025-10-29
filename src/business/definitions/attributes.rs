// Attribute business logic
// CRITICAL: This has NO knowledge of where/how data is stored

use crate::business::BusinessError;
use crate::business::validation::AttributeValidator;
use crate::models::definitions::AttributeDefinition;
use crate::models::common::AttributeId;
use std::sync::Arc;

// Import persistence trait (defined in persistence module)
// Business logic depends on trait, NOT implementation
use crate::persistence::AttributePersistence;

/// Trait for attribute business logic (injectable into API handlers)
#[async_trait::async_trait]
pub trait AttributeService: Send + Sync {
    /// Create a new attribute
    async fn create_attribute(&self, attr: AttributeDefinition) 
        -> Result<AttributeDefinition, BusinessError>;
    
    /// Update an existing attribute
    async fn update_attribute(&self, id: AttributeId, attr: AttributeDefinition) 
        -> Result<AttributeDefinition, BusinessError>;
    
    /// Delete an attribute
    async fn delete_attribute(&self, id: AttributeId) 
        -> Result<(), BusinessError>;
    
    /// Get a single attribute by ID
    async fn get_attribute(&self, id: AttributeId) 
        -> Result<AttributeDefinition, BusinessError>;
    
    /// List all attributes
    async fn list_all(&self) -> Result<Vec<AttributeDefinition>, BusinessError>;
    
    /// Search attributes by name (contains match)
    async fn search(&self, query: &str) 
        -> Result<Vec<AttributeDefinition>, BusinessError>;
}

/// Production implementation
/// CRITICAL: This has NO knowledge of where/how data is stored
pub struct AttributeServiceImpl {
    // Injected persistence - could be file, database, memory, network, etc.
    // Business logic doesn't know or care
    persistence: Arc<dyn AttributePersistence>,
    
    // Injected validator - also injectable for testing
    validator: Arc<dyn AttributeValidator>,
}

impl AttributeServiceImpl {
    /// Constructor accepts persistence via dependency injection
    /// Business logic NEVER creates its own persistence
    pub fn new(
        persistence: Arc<dyn AttributePersistence>,
        validator: Arc<dyn AttributeValidator>,
    ) -> Self {
        Self {
            persistence,
            validator,
        }
    }
}

#[async_trait::async_trait]
impl AttributeService for AttributeServiceImpl {
    async fn create_attribute(&self, attr: AttributeDefinition) 
        -> Result<AttributeDefinition, BusinessError> {
        // 1. Validate data integrity
        self.validator.validate(&attr, &[])?;
        
        // 2. Check uniqueness (using persistence abstraction)
        if self.persistence.exists_by_name(&attr.name).await? {
            return Err(BusinessError::DuplicateName(attr.name.clone()));
        }
        
        // 3. Persist (business logic doesn't know if this is file, DB, memory, etc.)
        self.persistence.save(&attr).await?;
        
        Ok(attr)
    }
    
    async fn update_attribute(&self, id: AttributeId, attr: AttributeDefinition) 
        -> Result<AttributeDefinition, BusinessError> {
        // 1. Ensure ID matches
        if attr.id.as_uuid() != id.as_uuid() {
            return Err(BusinessError::IdMismatch {
                expected: id.as_uuid().to_string(),
                actual: attr.id.as_uuid().to_string(),
            });
        }
        
        // 2. Check exists (using persistence abstraction)
        let existing = self.persistence.get(&id).await?
            .ok_or(BusinessError::NotFound)?;
        
        // 3. Validate data integrity
        self.validator.validate(&attr, &[])?;
        
        // 4. Check name uniqueness (if name changed)
        if attr.name.to_lowercase() != existing.name.to_lowercase()
            && self.persistence.exists_by_name(&attr.name).await? {
            return Err(BusinessError::DuplicateName(attr.name.clone()));
        }
        
        // 5. Persist (business logic has no idea how this works)
        self.persistence.save(&attr).await?;
        
        Ok(attr)
    }
    
    async fn delete_attribute(&self, id: AttributeId) 
        -> Result<(), BusinessError> {
        // Check exists (using persistence abstraction)
        if self.persistence.get(&id).await?.is_none() {
            return Err(BusinessError::NotFound);
        }
        
        // Delete (business logic doesn't know the implementation)
        self.persistence.delete(&id).await?;
        Ok(())
    }
    
    async fn get_attribute(&self, id: AttributeId) 
        -> Result<AttributeDefinition, BusinessError> {
        // Use persistence abstraction
        self.persistence.get(&id).await?
            .ok_or(BusinessError::NotFound)
    }
    
    async fn list_all(&self) -> Result<Vec<AttributeDefinition>, BusinessError> {
        // Use persistence abstraction
        Ok(self.persistence.list_all().await?)
    }
    
    async fn search(&self, query: &str) 
        -> Result<Vec<AttributeDefinition>, BusinessError> {
        // Use persistence abstraction (might be optimized in DB implementation)
        Ok(self.persistence.search_by_name(query).await?)
    }
}

// Verification that business logic doesn't know about storage:
// ✅ No `use std::fs` in this file
// ✅ No `use std::path::PathBuf` in this file
// ✅ No file paths in this file
// ✅ No JSON serialization in this file
// ✅ Only uses `AttributePersistence` trait
// ✅ Can swap implementations without changing code
