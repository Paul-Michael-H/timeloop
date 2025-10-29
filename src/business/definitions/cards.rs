// Card business logic service
// CRITICAL: Zero storage knowledge - uses injected persistence trait

use crate::models::cards::{CardDefinition, CardId};
use crate::persistence::traits::CardPersistence;
use crate::business::validation::CardValidator;
use crate::business::BusinessError;
use std::sync::Arc;

/// Service trait for card operations
#[async_trait::async_trait]
pub trait CardService: Send + Sync {
    async fn create_card(&self, card: CardDefinition) -> Result<CardDefinition, BusinessError>;
    async fn update_card(&self, card: CardDefinition) -> Result<CardDefinition, BusinessError>;
    async fn delete_card(&self, id: &CardId) -> Result<(), BusinessError>;
    async fn get_card(&self, id: &CardId) -> Result<CardDefinition, BusinessError>;
    async fn list_cards(&self) -> Result<Vec<CardDefinition>, BusinessError>;
    async fn search_cards(&self, query: &str) -> Result<Vec<CardDefinition>, BusinessError>;
}

/// Implementation of card service with dependency injection
pub struct CardServiceImpl<P: CardPersistence> {
    persistence: Arc<P>,
    validator: Arc<dyn CardValidator>,
}

impl<P: CardPersistence> CardServiceImpl<P> {
    pub fn new(persistence: Arc<P>, validator: Arc<dyn CardValidator>) -> Self {
        Self {
            persistence,
            validator,
        }
    }
}

#[async_trait::async_trait]
impl<P: CardPersistence> CardService for CardServiceImpl<P> {
    async fn create_card(&self, card: CardDefinition) -> Result<CardDefinition, BusinessError> {
        // Validate the card
        self.validator.validate(&card)
            .map_err(|e| BusinessError::ValidationError(e.to_string()))?;
        
        // Check for duplicate caption
        if self.persistence.exists_by_caption(&card.caption).await? {
            return Err(BusinessError::DuplicateName(card.caption.clone()));
        }
        
        // Save to persistence
        self.persistence.save(&card).await?;
        
        Ok(card)
    }
    
    async fn update_card(&self, card: CardDefinition) -> Result<CardDefinition, BusinessError> {
        // Validate the card
        self.validator.validate(&card)
            .map_err(|e| BusinessError::ValidationError(e.to_string()))?;
        
        // Check if card exists
        if self.persistence.get(&card.id).await?.is_none() {
            return Err(BusinessError::NotFound);
        }
        
        // Check for duplicate caption (excluding current card)
        let all_cards = self.persistence.list_all().await?;
        for existing in all_cards {
            if existing.id.to_string() != card.id.to_string() 
                && existing.caption.eq_ignore_ascii_case(&card.caption) {
                return Err(BusinessError::DuplicateName(card.caption.clone()));
            }
        }
        
        // Save to persistence
        self.persistence.save(&card).await?;
        
        Ok(card)
    }
    
    async fn delete_card(&self, id: &CardId) -> Result<(), BusinessError> {
        // Check if exists
        if self.persistence.get(id).await?.is_none() {
            return Err(BusinessError::NotFound);
        }
        
        // Delete from persistence
        self.persistence.delete(id).await?;
        
        Ok(())
    }
    
    async fn get_card(&self, id: &CardId) -> Result<CardDefinition, BusinessError> {
        self.persistence.get(id).await?
            .ok_or(BusinessError::NotFound)
    }
    
    async fn list_cards(&self) -> Result<Vec<CardDefinition>, BusinessError> {
        Ok(self.persistence.list_all().await?)
    }
    
    async fn search_cards(&self, query: &str) -> Result<Vec<CardDefinition>, BusinessError> {
        Ok(self.persistence.search_by_caption(query).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::memory_storage::InMemoryCardPersistence;
    use crate::business::validation::CardValidatorImpl;

    fn create_test_service() -> CardServiceImpl<InMemoryCardPersistence> {
        let persistence = Arc::new(InMemoryCardPersistence::new());
        let validator = Arc::new(CardValidatorImpl::new());
        CardServiceImpl::new(persistence, validator)
    }

    fn create_test_card(caption: &str) -> CardDefinition {
        let mut card = CardDefinition::new(caption.to_string());
        card.description = format!("Description for {}", caption);
        card
    }

    #[tokio::test]
    async fn test_create_card_success() {
        let service = create_test_service();
        let card = create_test_card("Fireball");
        
        let result = service.create_card(card.clone()).await;
        assert!(result.is_ok());
        
        let created = result.unwrap();
        assert_eq!(created.caption, "Fireball");
    }

    #[tokio::test]
    async fn test_create_duplicate_card_fails() {
        let service = create_test_service();
        let card1 = create_test_card("Fireball");
        let card2 = create_test_card("Fireball");
        
        service.create_card(card1).await.unwrap();
        let result = service.create_card(card2).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BusinessError::DuplicateName(_)));
    }

    #[tokio::test]
    async fn test_create_card_validates_empty_caption() {
        let service = create_test_service();
        let mut card = create_test_card("");
        card.description = "Valid description".to_string();
        
        let result = service.create_card(card).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BusinessError::ValidationError(_)));
    }

    #[tokio::test]
    async fn test_update_card_success() {
        let service = create_test_service();
        let mut card = create_test_card("Shield Block");
        
        let created = service.create_card(card.clone()).await.unwrap();
        
        card.id = created.id.clone();
        card.description = "Updated description".to_string();
        
        let result = service.update_card(card).await;
        assert!(result.is_ok());
        
        let updated = result.unwrap();
        assert_eq!(updated.description, "Updated description");
    }

    #[tokio::test]
    async fn test_update_nonexistent_card_fails() {
        let service = create_test_service();
        let card = create_test_card("Nonexistent");
        
        let result = service.update_card(card).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BusinessError::NotFound));
    }

    #[tokio::test]
    async fn test_delete_card_success() {
        let service = create_test_service();
        let card = create_test_card("Lightning Bolt");
        
        let created = service.create_card(card).await.unwrap();
        let result = service.delete_card(&created.id).await;
        
        assert!(result.is_ok());
        
        let get_result = service.get_card(&created.id).await;
        assert!(get_result.is_err());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_card_fails() {
        let service = create_test_service();
        let fake_id = CardId::new();
        
        let result = service.delete_card(&fake_id).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BusinessError::NotFound));
    }

    #[tokio::test]
    async fn test_list_cards() {
        let service = create_test_service();
        
        service.create_card(create_test_card("Card 1")).await.unwrap();
        service.create_card(create_test_card("Card 2")).await.unwrap();
        service.create_card(create_test_card("Card 3")).await.unwrap();
        
        let cards = service.list_cards().await.unwrap();
        assert_eq!(cards.len(), 3);
    }

    #[tokio::test]
    async fn test_search_cards() {
        let service = create_test_service();
        
        service.create_card(create_test_card("Fire Blast")).await.unwrap();
        service.create_card(create_test_card("Fireball")).await.unwrap();
        service.create_card(create_test_card("Ice Shard")).await.unwrap();
        
        let results = service.search_cards("fire").await.unwrap();
        assert_eq!(results.len(), 2);
        
        let results = service.search_cards("ice").await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_get_card_success() {
        let service = create_test_service();
        let card = create_test_card("Test Card");
        
        let created = service.create_card(card).await.unwrap();
        let fetched = service.get_card(&created.id).await.unwrap();
        
        assert_eq!(fetched.caption, "Test Card");
    }

    #[tokio::test]
    async fn test_get_card_not_found() {
        let service = create_test_service();
        let fake_id = CardId::new();
        
        let result = service.get_card(&fake_id).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BusinessError::NotFound));
    }
}
