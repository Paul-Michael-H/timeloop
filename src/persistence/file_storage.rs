// File-based persistence implementation for attributes
// Business logic doesn't know about this - it only knows the trait

use crate::persistence::traits::{AttributePersistence, PersistenceError};
use crate::models::definitions::AttributeDefinition;
use crate::models::common::AttributeId;
use std::path::PathBuf;
use std::sync::RwLock;
use tokio::fs;

/// File-based implementation (business logic doesn't know about this)
/// Stores attributes as JSON array in a single file
pub struct FileAttributePersistence {
    file_path: PathBuf,
    // Internal cache (optional optimization)
    cache: RwLock<Option<Vec<AttributeDefinition>>>,
}

impl FileAttributePersistence {
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        Self {
            file_path: file_path.into(),
            cache: RwLock::new(None),
        }
    }
    
    /// Internal helper - business logic never calls this
    async fn load_from_file(&self) -> Result<Vec<AttributeDefinition>, PersistenceError> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }
        
        let content = fs::read_to_string(&self.file_path).await?;
        let attrs: Vec<AttributeDefinition> = serde_json::from_str(&content)?;
        
        Ok(attrs)
    }
    
    /// Internal helper - business logic never calls this
    async fn save_to_file(&self, attrs: &[AttributeDefinition]) -> Result<(), PersistenceError> {
        // Create parent directories if they don't exist
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        // Create backup if file exists
        if self.file_path.exists() {
            let backup = self.file_path.with_extension("json.backup");
            // Ignore errors on backup (non-critical)
            let _ = fs::copy(&self.file_path, &backup).await;
        }
        
        // Serialize
        let json = serde_json::to_string_pretty(&attrs)?;
        
        // Atomic write (write to temp file, then rename)
        let temp_path = self.file_path.with_extension("tmp");
        fs::write(&temp_path, json).await?;
        fs::rename(&temp_path, &self.file_path).await?;
        
        // Update cache
        *self.cache.write().unwrap() = Some(attrs.to_vec());
        
        Ok(())
    }
}

#[async_trait::async_trait]
impl AttributePersistence for FileAttributePersistence {
    async fn save(&self, attr: &AttributeDefinition) -> Result<(), PersistenceError> {
        // Load all, upsert, save
        let mut all = self.list_all().await?;
        
        // Find and update or insert
        if let Some(existing) = all.iter_mut().find(|a| a.id.as_uuid() == attr.id.as_uuid()) {
            *existing = attr.clone();
        } else {
            all.push(attr.clone());
        }
        
        self.save_to_file(&all).await
    }
    
    async fn delete(&self, id: &AttributeId) -> Result<(), PersistenceError> {
        let mut all = self.list_all().await?;
        
        let initial_len = all.len();
        all.retain(|a| a.id.as_uuid() != id.as_uuid());
        
        if all.len() == initial_len {
            return Err(PersistenceError::NotFound);
        }
        
        self.save_to_file(&all).await
    }
    
    async fn get(&self, id: &AttributeId) -> Result<Option<AttributeDefinition>, PersistenceError> {
        let all = self.list_all().await?;
        Ok(all.into_iter().find(|a| a.id.as_uuid() == id.as_uuid()))
    }
    
    async fn list_all(&self) -> Result<Vec<AttributeDefinition>, PersistenceError> {
        // Check cache first
        if let Some(cached) = self.cache.read().unwrap().as_ref() {
            return Ok(cached.clone());
        }
        
        // Load from file
        let attrs = self.load_from_file().await?;
        
        // Update cache
        *self.cache.write().unwrap() = Some(attrs.clone());
        
        Ok(attrs)
    }
    
    async fn exists_by_name(&self, name: &str) -> Result<bool, PersistenceError> {
        let all = self.list_all().await?;
        let name_lower = name.to_lowercase();
        Ok(all.iter().any(|a| a.name.to_lowercase() == name_lower))
    }
    
    async fn search_by_name(&self, query: &str) -> Result<Vec<AttributeDefinition>, PersistenceError> {
        let all = self.list_all().await?;
        let query_lower = query.to_lowercase();
        
        Ok(all.into_iter()
            .filter(|a| a.name.to_lowercase().contains(&query_lower))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::common::Percentage;
    use crate::models::definitions::AttributeCategory;
    use tempfile::TempDir;
    
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
    async fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("attributes.json");
        let persistence = FileAttributePersistence::new(&file_path);
        
        let attr = create_test_attribute("Strength");
        
        // Save
        persistence.save(&attr).await.unwrap();
        
        // Verify file exists
        assert!(file_path.exists());
        
        // Load
        let loaded = persistence.get(&attr.id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().name, "Strength");
    }
    
    #[tokio::test]
    async fn test_update_existing() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("attributes.json");
        let persistence = FileAttributePersistence::new(&file_path);
        
        let mut attr = create_test_attribute("Strength");
        persistence.save(&attr).await.unwrap();
        
        // Update
        attr.description = "Updated description".to_string();
        persistence.save(&attr).await.unwrap();
        
        // Verify update
        let loaded = persistence.get(&attr.id).await.unwrap().unwrap();
        assert_eq!(loaded.description, "Updated description");
        
        // Verify only one item
        let all = persistence.list_all().await.unwrap();
        assert_eq!(all.len(), 1);
    }
    
    #[tokio::test]
    async fn test_delete() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("attributes.json");
        let persistence = FileAttributePersistence::new(&file_path);
        
        let attr = create_test_attribute("Strength");
        persistence.save(&attr).await.unwrap();
        
        // Delete
        persistence.delete(&attr.id).await.unwrap();
        
        // Verify deleted
        let loaded = persistence.get(&attr.id).await.unwrap();
        assert!(loaded.is_none());
    }
    
    #[tokio::test]
    async fn test_delete_nonexistent_fails() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("attributes.json");
        let persistence = FileAttributePersistence::new(&file_path);
        
        let id = AttributeId::new();
        let result = persistence.delete(&id).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PersistenceError::NotFound));
    }
    
    #[tokio::test]
    async fn test_list_all() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("attributes.json");
        let persistence = FileAttributePersistence::new(&file_path);
        
        let attr1 = create_test_attribute("Strength");
        let attr2 = create_test_attribute("Agility");
        let attr3 = create_test_attribute("Intelligence");
        
        persistence.save(&attr1).await.unwrap();
        persistence.save(&attr2).await.unwrap();
        persistence.save(&attr3).await.unwrap();
        
        let all = persistence.list_all().await.unwrap();
        assert_eq!(all.len(), 3);
    }
    
    #[tokio::test]
    async fn test_exists_by_name() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("attributes.json");
        let persistence = FileAttributePersistence::new(&file_path);
        
        let attr = create_test_attribute("Strength");
        persistence.save(&attr).await.unwrap();
        
        assert!(persistence.exists_by_name("Strength").await.unwrap());
        assert!(persistence.exists_by_name("STRENGTH").await.unwrap()); // Case insensitive
        assert!(!persistence.exists_by_name("Agility").await.unwrap());
    }
    
    #[tokio::test]
    async fn test_search_by_name() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("attributes.json");
        let persistence = FileAttributePersistence::new(&file_path);
        
        let attr1 = create_test_attribute("Physical Strength");
        let attr2 = create_test_attribute("Physical Agility");
        let attr3 = create_test_attribute("Mental Fortitude");
        
        persistence.save(&attr1).await.unwrap();
        persistence.save(&attr2).await.unwrap();
        persistence.save(&attr3).await.unwrap();
        
        let results = persistence.search_by_name("Physical").await.unwrap();
        assert_eq!(results.len(), 2);
        
        let results = persistence.search_by_name("mental").await.unwrap(); // Case insensitive
        assert_eq!(results.len(), 1);
    }
    
    #[tokio::test]
    async fn test_cache_works() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("attributes.json");
        let persistence = FileAttributePersistence::new(&file_path);
        
        let attr = create_test_attribute("Strength");
        persistence.save(&attr).await.unwrap();
        
        // First call loads from file
        let all1 = persistence.list_all().await.unwrap();
        
        // Second call should use cache
        let all2 = persistence.list_all().await.unwrap();
        
        assert_eq!(all1.len(), all2.len());
    }
}

// ============================================================================
// Card Persistence Implementation
// ============================================================================

use crate::persistence::traits::CardPersistence;
use crate::models::cards::{CardDefinition, CardId};

/// File-based implementation for cards
pub struct FileCardPersistence {
    file_path: PathBuf,
    cache: RwLock<Option<Vec<CardDefinition>>>,
}

impl FileCardPersistence {
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        Self {
            file_path: file_path.into(),
            cache: RwLock::new(None),
        }
    }
    
    async fn load_from_file(&self) -> Result<Vec<CardDefinition>, PersistenceError> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }
        
        let content = fs::read_to_string(&self.file_path).await?;
        let cards: Vec<CardDefinition> = serde_json::from_str(&content)?;
        
        Ok(cards)
    }
    
    async fn save_to_file(&self, cards: &[CardDefinition]) -> Result<(), PersistenceError> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        if self.file_path.exists() {
            let backup = self.file_path.with_extension("json.backup");
            let _ = fs::copy(&self.file_path, &backup).await;
        }
        
        let json = serde_json::to_string_pretty(&cards)?;
        let temp_path = self.file_path.with_extension("tmp");
        fs::write(&temp_path, json).await?;
        fs::rename(&temp_path, &self.file_path).await?;
        
        *self.cache.write().unwrap() = Some(cards.to_vec());
        
        Ok(())
    }
}

#[async_trait::async_trait]
impl CardPersistence for FileCardPersistence {
    async fn save(&self, card: &CardDefinition) -> Result<(), PersistenceError> {
        let mut all = self.list_all().await?;
        
        if let Some(existing) = all.iter_mut().find(|c| c.id.to_string() == card.id.to_string()) {
            *existing = card.clone();
        } else {
            all.push(card.clone());
        }
        
        self.save_to_file(&all).await
    }
    
    async fn delete(&self, id: &CardId) -> Result<(), PersistenceError> {
        let mut all = self.list_all().await?;
        
        let initial_len = all.len();
        all.retain(|c| c.id.to_string() != id.to_string());
        
        if all.len() == initial_len {
            return Err(PersistenceError::NotFound);
        }
        
        self.save_to_file(&all).await
    }
    
    async fn get(&self, id: &CardId) -> Result<Option<CardDefinition>, PersistenceError> {
        let all = self.list_all().await?;
        Ok(all.into_iter().find(|c| c.id.to_string() == id.to_string()))
    }
    
    async fn list_all(&self) -> Result<Vec<CardDefinition>, PersistenceError> {
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(cached) = cache.as_ref() {
                return Ok(cached.clone());
            }
        }
        
        // Load from file
        let cards = self.load_from_file().await?;
        
        // Update cache
        {
            let mut cache = self.cache.write().unwrap();
            *cache = Some(cards.clone());
        }
        
        Ok(cards)
    }
    
    async fn exists_by_caption(&self, caption: &str) -> Result<bool, PersistenceError> {
        let all = self.list_all().await?;
        Ok(all.iter().any(|c| c.caption.eq_ignore_ascii_case(caption)))
    }
    
    async fn search_by_caption(&self, query: &str) -> Result<Vec<CardDefinition>, PersistenceError> {
        let all = self.list_all().await?;
        let query_lower = query.to_lowercase();
        
        Ok(all.into_iter()
            .filter(|c| c.caption.to_lowercase().contains(&query_lower))
            .collect())
    }
}
