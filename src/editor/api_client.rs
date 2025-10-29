// API client for editor - communicates with server
// Replaces direct file I/O with REST API calls

use crate::models::definitions::AttributeDefinition;
use crate::models::common::AttributeId;
use reqwest;

/// API client for editor operations
#[derive(Clone)]
pub struct EditorApiClient {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Debug)]
pub enum ApiError {
    NetworkError(String),
    ServerError(u16, String),
    DeserializationError(String),
    NotFound,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ApiError::ServerError(code, msg) => write!(f, "Server error {}: {}", code, msg),
            ApiError::DeserializationError(msg) => write!(f, "Failed to parse response: {}", msg),
            ApiError::NotFound => write!(f, "Resource not found"),
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::NetworkError(err.to_string())
    }
}

impl EditorApiClient {
    /// Create a new API client
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
        }
    }
    
    /// Create attribute definition
    pub async fn create_attribute(&self, attr: AttributeDefinition) 
        -> Result<AttributeDefinition, ApiError> {
        let url = format!("{}/api/definitions/attributes", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&attr)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ApiError::ServerError(status, text));
        }
        
        let created: AttributeDefinition = response.json().await
            .map_err(|e| ApiError::DeserializationError(e.to_string()))?;
        
        Ok(created)
    }
    
    /// Update attribute definition
    pub async fn update_attribute(&self, id: AttributeId, attr: AttributeDefinition) 
        -> Result<AttributeDefinition, ApiError> {
        let url = format!("{}/api/definitions/attributes/{}", self.base_url, id.as_uuid());
        
        let response = self.client
            .put(&url)
            .json(&attr)
            .send()
            .await?;
        
        if response.status() == 404 {
            return Err(ApiError::NotFound);
        }
        
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ApiError::ServerError(status, text));
        }
        
        let updated: AttributeDefinition = response.json().await
            .map_err(|e| ApiError::DeserializationError(e.to_string()))?;
        
        Ok(updated)
    }
    
    /// Delete attribute definition
    pub async fn delete_attribute(&self, id: AttributeId) 
        -> Result<(), ApiError> {
        let url = format!("{}/api/definitions/attributes/{}", self.base_url, id.as_uuid());
        
        let response = self.client
            .delete(&url)
            .send()
            .await?;
        
        if response.status() == 404 {
            return Err(ApiError::NotFound);
        }
        
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ApiError::ServerError(status, text));
        }
        
        Ok(())
    }
    
    /// Get single attribute definition
    pub async fn get_attribute(&self, id: AttributeId) 
        -> Result<AttributeDefinition, ApiError> {
        let url = format!("{}/api/definitions/attributes/{}", self.base_url, id.as_uuid());
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if response.status() == 404 {
            return Err(ApiError::NotFound);
        }
        
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ApiError::ServerError(status, text));
        }
        
        let attr: AttributeDefinition = response.json().await
            .map_err(|e| ApiError::DeserializationError(e.to_string()))?;
        
        Ok(attr)
    }
    
    /// List all attribute definitions
    pub async fn list_attributes(&self) 
        -> Result<Vec<AttributeDefinition>, ApiError> {
        let url = format!("{}/api/definitions/attributes", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ApiError::ServerError(status, text));
        }
        
        let attrs: Vec<AttributeDefinition> = response.json().await
            .map_err(|e| ApiError::DeserializationError(e.to_string()))?;
        
        Ok(attrs)
    }
    
    /// Search attribute definitions
    pub async fn search_attributes(&self, query: &str) 
        -> Result<Vec<AttributeDefinition>, ApiError> {
        let url = format!("{}/api/definitions/attributes?q={}", self.base_url, query);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ApiError::ServerError(status, text));
        }
        
        let attrs: Vec<AttributeDefinition> = response.json().await
            .map_err(|e| ApiError::DeserializationError(e.to_string()))?;
        
        Ok(attrs)
    }
    
    /// Health check - verify server is reachable
    pub async fn health_check(&self) -> Result<bool, ApiError> {
        let url = format!("{}/health", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        Ok(response.status().is_success())
    }
    
    // ========================================================================
    // CARD API METHODS
    // ========================================================================
    
    /// Create card definition
    pub async fn create_card(&self, card: crate::models::cards::CardDefinition) 
        -> Result<crate::models::cards::CardDefinition, ApiError> {
        let url = format!("{}/api/definitions/cards", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&card)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ApiError::ServerError(status, text));
        }
        
        let created: crate::models::cards::CardDefinition = response.json().await
            .map_err(|e| ApiError::DeserializationError(e.to_string()))?;
        
        Ok(created)
    }
    
    /// Update card definition
    pub async fn update_card(&self, id: crate::models::cards::CardId, card: crate::models::cards::CardDefinition) 
        -> Result<crate::models::cards::CardDefinition, ApiError> {
        let url = format!("{}/api/definitions/cards/{}", self.base_url, id);
        
        let response = self.client
            .put(&url)
            .json(&card)
            .send()
            .await?;
        
        if response.status() == 404 {
            return Err(ApiError::NotFound);
        }
        
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ApiError::ServerError(status, text));
        }
        
        let updated: crate::models::cards::CardDefinition = response.json().await
            .map_err(|e| ApiError::DeserializationError(e.to_string()))?;
        
        Ok(updated)
    }
    
    /// Delete card definition
    pub async fn delete_card(&self, id: crate::models::cards::CardId) 
        -> Result<(), ApiError> {
        let url = format!("{}/api/definitions/cards/{}", self.base_url, id);
        
        let response = self.client
            .delete(&url)
            .send()
            .await?;
        
        if response.status() == 404 {
            return Err(ApiError::NotFound);
        }
        
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ApiError::ServerError(status, text));
        }
        
        Ok(())
    }
    
    /// Get single card definition
    pub async fn get_card(&self, id: crate::models::cards::CardId) 
        -> Result<crate::models::cards::CardDefinition, ApiError> {
        let url = format!("{}/api/definitions/cards/{}", self.base_url, id);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if response.status() == 404 {
            return Err(ApiError::NotFound);
        }
        
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ApiError::ServerError(status, text));
        }
        
        let card: crate::models::cards::CardDefinition = response.json().await
            .map_err(|e| ApiError::DeserializationError(e.to_string()))?;
        
        Ok(card)
    }
    
    /// List all card definitions
    pub async fn list_cards(&self) 
        -> Result<Vec<crate::models::cards::CardDefinition>, ApiError> {
        let url = format!("{}/api/definitions/cards", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ApiError::ServerError(status, text));
        }
        
        let cards: Vec<crate::models::cards::CardDefinition> = response.json().await
            .map_err(|e| ApiError::DeserializationError(e.to_string()))?;
        
        Ok(cards)
    }
    
    /// Search card definitions
    pub async fn search_cards(&self, query: &str) 
        -> Result<Vec<crate::models::cards::CardDefinition>, ApiError> {
        let url = format!("{}/api/definitions/cards?q={}", self.base_url, query);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ApiError::ServerError(status, text));
        }
        
        let cards: Vec<crate::models::cards::CardDefinition> = response.json().await
            .map_err(|e| ApiError::DeserializationError(e.to_string()))?;
        
        Ok(cards)
    }
}
