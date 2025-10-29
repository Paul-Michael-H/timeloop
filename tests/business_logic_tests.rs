// Business Logic Tests - Testing services with in-memory persistence
// These tests verify business logic WITHOUT touching the file system or HTTP

use timeloop::business::definitions::AttributeServiceImpl;
use timeloop::business::validation::AttributeValidatorImpl;
use timeloop::business::{AttributeService, BusinessError};
use timeloop::persistence::{InMemoryAttributePersistence, AttributePersistence};
use timeloop::models::definitions::{AttributeDefinition, AttributeCategory};
use timeloop::models::common::{AttributeId, Percentage};
use std::sync::Arc;

/// Helper function to create a valid test attribute
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

/// Helper function to create a service with in-memory persistence
fn create_test_service() -> AttributeServiceImpl {
    let persistence = Arc::new(InMemoryAttributePersistence::new());
    let validator = Arc::new(AttributeValidatorImpl::new());
    AttributeServiceImpl::new(persistence, validator)
}

#[tokio::test]
async fn test_create_attribute_success() {
    let service = create_test_service();
    let attr = create_test_attribute("Strength");
    
    let result = service.create_attribute(attr.clone()).await;
    
    assert!(result.is_ok());
    let created = result.unwrap();
    assert_eq!(created.name, "Strength");
}

#[tokio::test]
async fn test_create_attribute_validates_empty_name() {
    let service = create_test_service();
    let mut attr = create_test_attribute("Strength");
    attr.name = "".to_string();
    
    let result = service.create_attribute(attr).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BusinessError::ValidationError(_)));
}

#[tokio::test]
async fn test_create_attribute_prevents_duplicate_names() {
    let service = create_test_service();
    let attr1 = create_test_attribute("Strength");
    let attr2 = create_test_attribute("Strength");
    
    // First create should succeed
    service.create_attribute(attr1).await.unwrap();
    
    // Second create with same name should fail
    let result = service.create_attribute(attr2).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BusinessError::DuplicateName(_)));
}

#[tokio::test]
async fn test_create_attribute_case_insensitive_duplicates() {
    let service = create_test_service();
    let attr1 = create_test_attribute("Strength");
    let attr2 = create_test_attribute("STRENGTH");
    
    service.create_attribute(attr1).await.unwrap();
    
    let result = service.create_attribute(attr2).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BusinessError::DuplicateName(_)));
}

#[tokio::test]
async fn test_get_attribute_success() {
    let service = create_test_service();
    let attr = create_test_attribute("Strength");
    let id = attr.id;
    
    service.create_attribute(attr).await.unwrap();
    
    let result = service.get_attribute(id).await;
    
    assert!(result.is_ok());
    let retrieved = result.unwrap();
    assert_eq!(retrieved.name, "Strength");
    assert_eq!(retrieved.id.as_uuid(), id.as_uuid());
}

#[tokio::test]
async fn test_get_attribute_not_found() {
    let service = create_test_service();
    let id = AttributeId::new();
    
    let result = service.get_attribute(id).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BusinessError::NotFound));
}

#[tokio::test]
async fn test_update_attribute_success() {
    let service = create_test_service();
    let mut attr = create_test_attribute("Strength");
    let id = attr.id;
    
    service.create_attribute(attr.clone()).await.unwrap();
    
    // Update the attribute
    attr.description = "Updated description".to_string();
    attr.base_value = 20;
    
    let result = service.update_attribute(id, attr).await;
    
    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.description, "Updated description");
    assert_eq!(updated.base_value, 20);
}

#[tokio::test]
async fn test_update_attribute_not_found() {
    let service = create_test_service();
    let attr = create_test_attribute("Strength");
    
    let result = service.update_attribute(attr.id, attr).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BusinessError::NotFound));
}

#[tokio::test]
async fn test_update_attribute_id_mismatch() {
    let service = create_test_service();
    let attr = create_test_attribute("Strength");
    service.create_attribute(attr.clone()).await.unwrap();
    
    let different_id = AttributeId::new();
    let result = service.update_attribute(different_id, attr).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BusinessError::IdMismatch { .. }));
}

#[tokio::test]
async fn test_update_attribute_validates() {
    let service = create_test_service();
    let mut attr = create_test_attribute("Strength");
    service.create_attribute(attr.clone()).await.unwrap();
    
    // Invalid update (empty name)
    attr.name = "".to_string();
    let result = service.update_attribute(attr.id, attr).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BusinessError::ValidationError(_)));
}

#[tokio::test]
async fn test_delete_attribute_success() {
    let service = create_test_service();
    let attr = create_test_attribute("Strength");
    let id = attr.id;
    
    service.create_attribute(attr).await.unwrap();
    
    let result = service.delete_attribute(id).await;
    
    assert!(result.is_ok());
    
    // Verify it's really deleted
    let get_result = service.get_attribute(id).await;
    assert!(get_result.is_err());
}

#[tokio::test]
async fn test_delete_attribute_not_found() {
    let service = create_test_service();
    let id = AttributeId::new();
    
    let result = service.delete_attribute(id).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BusinessError::NotFound));
}

#[tokio::test]
async fn test_list_all_attributes() {
    let service = create_test_service();
    
    service.create_attribute(create_test_attribute("Strength")).await.unwrap();
    service.create_attribute(create_test_attribute("Agility")).await.unwrap();
    service.create_attribute(create_test_attribute("Intelligence")).await.unwrap();
    
    let result = service.list_all().await;
    
    assert!(result.is_ok());
    let all = result.unwrap();
    assert_eq!(all.len(), 3);
}

#[tokio::test]
async fn test_search_attributes() {
    let service = create_test_service();
    
    service.create_attribute(create_test_attribute("Physical Strength")).await.unwrap();
    service.create_attribute(create_test_attribute("Physical Agility")).await.unwrap();
    service.create_attribute(create_test_attribute("Mental Fortitude")).await.unwrap();
    
    let result = service.search("Physical").await;
    
    assert!(result.is_ok());
    let found = result.unwrap();
    assert_eq!(found.len(), 2);
    assert!(found.iter().all(|a| a.name.contains("Physical")));
}

#[tokio::test]
async fn test_search_case_insensitive() {
    let service = create_test_service();
    
    service.create_attribute(create_test_attribute("Strength")).await.unwrap();
    
    let result = service.search("STRENGTH").await;
    
    assert!(result.is_ok());
    let found = result.unwrap();
    assert_eq!(found.len(), 1);
}

#[tokio::test]
async fn test_full_crud_cycle() {
    let service = create_test_service();
    
    // Create
    let mut attr = create_test_attribute("Strength");
    let created = service.create_attribute(attr.clone()).await.unwrap();
    assert_eq!(created.name, "Strength");
    
    // Read
    let retrieved = service.get_attribute(created.id).await.unwrap();
    assert_eq!(retrieved.name, "Strength");
    
    // Update
    attr.id = created.id;
    attr.description = "Updated".to_string();
    let updated = service.update_attribute(created.id, attr).await.unwrap();
    assert_eq!(updated.description, "Updated");
    
    // List
    let all = service.list_all().await.unwrap();
    assert_eq!(all.len(), 1);
    
    // Delete
    service.delete_attribute(created.id).await.unwrap();
    let all = service.list_all().await.unwrap();
    assert_eq!(all.len(), 0);
}

#[tokio::test]
async fn test_business_logic_uses_persistence_abstraction() {
    // This test verifies that business logic doesn't know about storage
    // We can swap persistence implementations without changing business logic
    
    // Create service with in-memory persistence
    let memory_persistence = Arc::new(InMemoryAttributePersistence::new());
    let validator = Arc::new(AttributeValidatorImpl::new());
    let service = AttributeServiceImpl::new(memory_persistence.clone(), validator);
    
    // Create attribute through service
    let attr = create_test_attribute("Test");
    service.create_attribute(attr.clone()).await.unwrap();
    
    // Verify it was stored (business logic successfully used abstraction)
    let stored = memory_persistence.get(&attr.id).await.unwrap();
    assert!(stored.is_some());
    assert_eq!(stored.unwrap().name, "Test");
}

#[tokio::test]
async fn test_validation_errors_are_caught() {
    let service = create_test_service();
    
    // Test various validation errors
    let mut attr = create_test_attribute("Test");
    
    // Empty name
    attr.name = "".to_string();
    assert!(service.create_attribute(attr.clone()).await.is_err());
    
    // Invalid range
    attr.name = "Test".to_string();
    attr.min_value = 100;
    attr.max_value = 50;
    assert!(service.create_attribute(attr.clone()).await.is_err());
    
    // Base value out of range
    attr.min_value = 1;
    attr.max_value = 100;
    attr.base_value = 200;
    assert!(service.create_attribute(attr.clone()).await.is_err());
    
    // Zero training difficulty
    attr.base_value = 10;
    attr.training_difficulty = Percentage::new(0);
    assert!(service.create_attribute(attr).await.is_err());
}
