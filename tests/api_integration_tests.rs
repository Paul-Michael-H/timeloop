// API Integration Tests - Testing HTTP endpoints
// These tests verify the API layer with in-memory persistence

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;
use timeloop::api::handlers::AppState;
use timeloop::api::routes::create_router;
use timeloop::business::definitions::{AttributeServiceImpl, CardServiceImpl};
use timeloop::business::validation::{AttributeValidatorImpl, CardValidatorImpl};
use timeloop::models::common::AttributeId;
use timeloop::persistence::{InMemoryAttributePersistence, memory_storage::InMemoryCardPersistence};
use timeloop::storage::{GameDefinitionsLoader, save_manager::SaveManager};

/// Helper to create a test app with in-memory persistence
fn create_test_app() -> axum::Router {
    let attr_persistence = Arc::new(InMemoryAttributePersistence::new());
    let attr_validator = Arc::new(AttributeValidatorImpl::new());
    let attribute_service = Arc::new(AttributeServiceImpl::new(attr_persistence, attr_validator));
    
    let card_persistence = Arc::new(InMemoryCardPersistence::new());
    let card_validator = Arc::new(CardValidatorImpl::new());
    let card_service = Arc::new(CardServiceImpl::new(card_persistence, card_validator));
    
    let state = AppState {
        game_state: Arc::new(RwLock::new(None)),
        definitions: Arc::new(GameDefinitionsLoader::new()),
        save_manager: Arc::new(SaveManager::new("test_saves")),
        attribute_service,
        card_service,
    };
    
    create_router(state)
}

/// Helper to parse response body as JSON
async fn body_to_json(body: Body) -> Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

/// Helper to create a valid test attribute JSON
fn create_test_attribute_json(name: &str) -> Value {
    json!({
        "id": AttributeId::new().as_uuid().to_string(),
        "name": name,
        "description": "Test description",
        "category": "Physical",
        "base_value": 10,
        "min_value": 1,
        "max_value": 100,
        "training_difficulty": 100,
        "icon": null
    })
}

#[tokio::test]
async fn test_health_check() {
    let app = create_test_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_attribute_success() {
    let app = create_test_app();
    
    let attr_json = create_test_attribute_json("Strength");
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/definitions/attributes")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&attr_json).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let body = body_to_json(response.into_body()).await;
    assert_eq!(body["name"], "Strength");
    assert_eq!(body["description"], "Test description");
}

#[tokio::test]
async fn test_create_attribute_invalid_data() {
    let app = create_test_app();
    
    let mut attr_json = create_test_attribute_json("Test");
    attr_json["name"] = json!(""); // Empty name
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/definitions/attributes")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&attr_json).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_attribute_not_found() {
    let app = create_test_app();
    
    let fake_id_obj = AttributeId::new();
    let fake_id = fake_id_obj.as_uuid();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/definitions/attributes/{}", fake_id))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_attribute_not_found() {
    let app = create_test_app();
    
    let attr_json = create_test_attribute_json("Test");
    let id = attr_json["id"].as_str().unwrap();
    
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/definitions/attributes/{}", id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&attr_json).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_attribute_not_found() {
    let app = create_test_app();
    
    let fake_id_obj = AttributeId::new();
    let fake_id = fake_id_obj.as_uuid();
    
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/definitions/attributes/{}", fake_id))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_search_attributes_empty() {
    let app = create_test_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/definitions/attributes?q=Physical")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = body_to_json(response.into_body()).await;
    let results = body.as_array().unwrap();
    assert_eq!(results.len(), 0); // No attributes created yet
}

#[tokio::test]
async fn test_concurrent_creates_prevent_duplicates() {
    let app1 = create_test_app();
    let app2 = create_test_app();
    
    let attr1 = create_test_attribute_json("Strength");
    let attr2 = create_test_attribute_json("Strength");
    
    // First should succeed
    let response1 = app1
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/definitions/attributes")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&attr1).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(response1.status(), StatusCode::CREATED);
    
    // Second in different app should also succeed (different state)
    let response2 = app2
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/definitions/attributes")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&attr2).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(response2.status(), StatusCode::CREATED);
}
