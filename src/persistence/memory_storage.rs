// In-memory persistence implementation for testing
// Business logic doesn't know about this either - same trait as file storage

use crate::persistence::traits::{AttributePersistence, PersistenceError};
use crate::models::definitions::AttributeDefinition;
use crate::models::common::AttributeId;
use std::collections::HashMap;
use std::sync::RwLock;

/// In-memory implementation for testing (business logic doesn't know about this)
/// No file I/O - everything in memory
pub struct InMemoryAttributePersistence {
    storage: RwLock<HashMap<AttributeId, AttributeDefinition>>,
}

impl InMemoryAttributePersistence {
    pub fn new() -> Self {
        Self {
            storage: RwLock::new(HashMap::new()),
        }
    }
    
    pub fn with_data(data: Vec<AttributeDefinition>) -> Self {
        let mut map = HashMap::new();
        for attr in data {
            map.insert(attr.id, attr);
        }
        
        Self {
            storage: RwLock::new(map),
        }
    }
}

impl Default for InMemoryAttributePersistence {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AttributePersistence for InMemoryAttributePersistence {
    async fn save(&self, attr: &AttributeDefinition) -> Result<(), PersistenceError> {
        self.storage.write().unwrap().insert(attr.id, attr.clone());
        Ok(())
    }
    
    async fn delete(&self, id: &AttributeId) -> Result<(), PersistenceError> {
        self.storage.write().unwrap()
            .remove(id)
            .ok_or(PersistenceError::NotFound)?;
        Ok(())
    }
    
    async fn get(&self, id: &AttributeId) -> Result<Option<AttributeDefinition>, PersistenceError> {
        Ok(self.storage.read().unwrap().get(id).cloned())
    }
    
    async fn list_all(&self) -> Result<Vec<AttributeDefinition>, PersistenceError> {
        Ok(self.storage.read().unwrap().values().cloned().collect())
    }
    
    async fn exists_by_name(&self, name: &str) -> Result<bool, PersistenceError> {
        let name_lower = name.to_lowercase();
        Ok(self.storage.read().unwrap()
            .values()
            .any(|a| a.name.to_lowercase() == name_lower))
    }
    
    async fn search_by_name(&self, query: &str) -> Result<Vec<AttributeDefinition>, PersistenceError> {
        let query_lower = query.to_lowercase();
        Ok(self.storage.read().unwrap()
            .values()
            .filter(|a| a.name.to_lowercase().contains(&query_lower))
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::common::Percentage;
    use crate::models::definitions::AttributeCategory;
    
    fn create_test_attribute(name: &str) -> AttributeDefinition {
        AttributeDefinition {
            id: AttributeId::new(),
            name: name.to_string(),
            description: "Test description".to_string(),
            category: AttributeCategory::Physical,
            base_value: 10,
            min_value: 1,
            max_value: 100,
            training_difficulty: Percentage::new(100),
            icon: None,
        }
    }
    
    #[tokio::test]
    async fn test_save_and_get() {
        let persistence = InMemoryAttributePersistence::new();
        let attr = create_test_attribute("Strength");
        
        persistence.save(&attr).await.unwrap();
        
        let loaded = persistence.get(&attr.id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().name, "Strength");
    }
    
    #[tokio::test]
    async fn test_update_existing() {
        let persistence = InMemoryAttributePersistence::new();
        let mut attr = create_test_attribute("Strength");
        
        persistence.save(&attr).await.unwrap();
        
        // Update
        attr.description = "Updated".to_string();
        persistence.save(&attr).await.unwrap();
        
        let loaded = persistence.get(&attr.id).await.unwrap().unwrap();
        assert_eq!(loaded.description, "Updated");
    }
    
    #[tokio::test]
    async fn test_delete() {
        let persistence = InMemoryAttributePersistence::new();
        let attr = create_test_attribute("Strength");
        
        persistence.save(&attr).await.unwrap();
        persistence.delete(&attr.id).await.unwrap();
        
        let loaded = persistence.get(&attr.id).await.unwrap();
        assert!(loaded.is_none());
    }
    
    #[tokio::test]
    async fn test_delete_not_found() {
        let persistence = InMemoryAttributePersistence::new();
        let id = AttributeId::new();
        
        let result = persistence.delete(&id).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_list_all() {
        let persistence = InMemoryAttributePersistence::new();
        
        persistence.save(&create_test_attribute("Strength")).await.unwrap();
        persistence.save(&create_test_attribute("Agility")).await.unwrap();
        persistence.save(&create_test_attribute("Intelligence")).await.unwrap();
        
        let all = persistence.list_all().await.unwrap();
        assert_eq!(all.len(), 3);
    }
    
    #[tokio::test]
    async fn test_exists_by_name() {
        let persistence = InMemoryAttributePersistence::new();
        let attr = create_test_attribute("Strength");
        
        persistence.save(&attr).await.unwrap();
        
        assert!(persistence.exists_by_name("Strength").await.unwrap());
        assert!(persistence.exists_by_name("STRENGTH").await.unwrap());
        assert!(!persistence.exists_by_name("Agility").await.unwrap());
    }
    
    #[tokio::test]
    async fn test_search() {
        let persistence = InMemoryAttributePersistence::new();
        
        persistence.save(&create_test_attribute("Physical Strength")).await.unwrap();
        persistence.save(&create_test_attribute("Physical Agility")).await.unwrap();
        persistence.save(&create_test_attribute("Mental Fortitude")).await.unwrap();
        
        let results = persistence.search_by_name("Physical").await.unwrap();
        assert_eq!(results.len(), 2);
        
        let results = persistence.search_by_name("mental").await.unwrap();
        assert_eq!(results.len(), 1);
    }
    
    #[tokio::test]
    async fn test_with_data() {
        let attrs = vec![
            create_test_attribute("Strength"),
            create_test_attribute("Agility"),
        ];
        
        let persistence = InMemoryAttributePersistence::with_data(attrs);
        
        let all = persistence.list_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }
}
