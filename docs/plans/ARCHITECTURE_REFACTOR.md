# Architecture Refactor Plan - Client-Server Pattern

## Overview
Refactor the Timeloop architecture to follow a consistent client-server pattern where all clients (Game Client and Editor) communicate exclusively through REST API, with the server handling business logic and persistence.

## Core Architectural Principles

### 1. **No Backward Compatibility Required**
- Clean slate refactor - no need to support old file-based approach
- Break existing editor/client immediately
- Can change data formats freely
- Backward compatibility only added when explicitly required

### 2. **Pure Rust Stack**
- All components written in Rust
- Clients: Bevy + egui
- Server: Axum + Tokio
- Persistence: File system (JSON) with trait abstraction
- No web technologies, no JavaScript, no separate tech stacks

### 3. **Dependency Injection Pattern**
- API handlers are **thin wrappers only**
- All business logic in injectable services
- API Server has **zero business logic**
- Services are trait-based for easy testing
- Mock all dependencies in tests

## Current Architecture Problems

### Editor (Current)
- **Direct file system access**: Editor writes directly to `game_data/core/attributes.json`
- **No server validation**: Changes bypass server business logic
- **No multi-client support**: Two editors can't work simultaneously
- **Testing complexity**: Can't test without file system

### Game Client (Current - Partially Good)
- **Mixed approach**: Uses API for gameplay but needs improvement
- **No CRUD operations**: Can't create/edit/delete game objects
- **Initialization complexity**: Loads definitions on startup instead of from API

### Server (Current)
- **Read-only for definitions**: No endpoints to modify attributes/affinities/effects
- **File system coupled**: Loads directly from JSON files at startup
- **No business logic layer**: Validation mixed with API handlers

## Target Architecture

```
┌─────────────────────┐         ┌─────────────────────┐
│   Game Client       │         │   Editor Client     │
│   (Bevy + egui)     │         │   (Bevy + egui)     │
└──────────┬──────────┘         └──────────┬──────────┘
           │                               │
           │ HTTP REST API                 │ HTTP REST API
           │                               │
           └───────────────┬───────────────┘
                           │
                  ┌────────▼────────┐
                  │   API Server    │
                  │   (Axum)        │
                  │                 │
                  │  - Routes       │
                  │  - Auth         │
                  │  - Validation   │
                  └────────┬────────┘
                           │
                  ┌────────▼────────┐
                  │ Business Logic  │
                  │                 │
                  │ - Game Engine   │
                  │ - Definitions   │
                  │ - Validation    │
                  │ - State Mgmt    │
                  └────────┬────────┘
                           │
                  ┌────────▼────────┐
                  │  Persistence    │
                  │                 │
                  │ - File Storage  │
                  │ - Save Manager  │
                  │ - Data Loader   │
                  └─────────────────┘
```

## Implementation Plan

### Phase 1: Server Refactoring (Foundation)

#### 1.1 Create Business Logic Layer (2 hours)
**File**: `src/business/mod.rs` (new module)

```rust
src/
├── business/
│   ├── mod.rs              // Module exports
│   ├── definitions/
│   │   ├── mod.rs
│   │   ├── attributes.rs   // Attribute CRUD logic
│   │   ├── affinities.rs   // Affinity CRUD logic
│   │   └── effects.rs      // Effect CRUD logic
│   ├── validation/
│   │   ├── mod.rs
│   │   ├── attribute_validator.rs
│   │   ├── affinity_validator.rs
│   │   └── effect_validator.rs
│   └── game_state/
│       ├── mod.rs
│       ├── character.rs    // Character business logic
│       └── progression.rs  // Training/progression logic
```

**Key responsibilities**:
- All CRUD operations for definitions
- All validation logic (moved from editor)
- Game state management
- Business rules enforcement
- **No direct file system access** - uses persistence layer

**Example API** (trait-based for dependency injection):
```rust
// src/business/definitions/attributes.rs

/// Trait for attribute business logic (injectable into API handlers)
pub trait AttributeService: Send + Sync {
    fn create_attribute(&self, attr: AttributeDefinition) 
        -> Result<AttributeDefinition, BusinessError>;
    
    fn update_attribute(&self, id: AttributeId, attr: AttributeDefinition) 
        -> Result<AttributeDefinition, BusinessError>;
    
    fn delete_attribute(&self, id: AttributeId) 
        -> Result<(), BusinessError>;
    
    fn get_attribute(&self, id: AttributeId) 
        -> Result<AttributeDefinition, BusinessError>;
    
    fn list_all(&self) -> Result<Vec<AttributeDefinition>, BusinessError>;
    
    fn search(&self, query: &str) 
        -> Result<Vec<AttributeDefinition>, BusinessError>;
}

/// Production implementation
pub struct AttributeServiceImpl {
    persistence: Arc<dyn AttributePersistence>,
    validator: Arc<dyn AttributeValidator>,
}

impl AttributeService for AttributeServiceImpl {
    fn create_attribute(&self, attr: AttributeDefinition) 
        -> Result<AttributeDefinition, BusinessError> {
        // 1. Validate
        self.validator.validate(&attr, &self.list_all()?)?;
        
        // 2. Check uniqueness
        if self.persistence.exists_by_name(&attr.name)? {
            return Err(BusinessError::DuplicateName);
        }
        
        // 3. Persist
        self.persistence.save(&attr)?;
        
        Ok(attr)
    }
    
    fn update_attribute(&self, id: AttributeId, attr: AttributeDefinition) 
        -> Result<AttributeDefinition, BusinessError> {
        // 1. Ensure ID matches
        if attr.id != id {
            return Err(BusinessError::IdMismatch);
        }
        
        // 2. Check exists
        if self.persistence.get(&id)?.is_none() {
            return Err(BusinessError::NotFound);
        }
        
        // 3. Validate
        self.validator.validate(&attr, &self.list_all()?)?;
        
        // 4. Persist
        self.persistence.save(&attr)?;
        
        Ok(attr)
    }
    
    fn delete_attribute(&self, id: AttributeId) 
        -> Result<(), BusinessError> {
        // Check exists
        if self.persistence.get(&id)?.is_none() {
            return Err(BusinessError::NotFound);
        }
        
        self.persistence.delete(&id)?;
        Ok(())
    }
    
    fn get_attribute(&self, id: AttributeId) 
        -> Result<AttributeDefinition, BusinessError> {
        self.persistence.get(&id)?
            .ok_or(BusinessError::NotFound)
    }
    
    fn list_all(&self) -> Result<Vec<AttributeDefinition>, BusinessError> {
        Ok(self.persistence.list_all()?)
    }
    
    fn search(&self, query: &str) 
        -> Result<Vec<AttributeDefinition>, BusinessError> {
        let all = self.list_all()?;
        let query_lower = query.to_lowercase();
        
        Ok(all.into_iter()
            .filter(|a| a.name.to_lowercase().contains(&query_lower))
            .collect())
    }
}
```

#### 1.2 Create Persistence Layer Traits (1 hour)
**File**: `src/persistence/mod.rs` (refactor existing storage)

```rust
// src/persistence/traits.rs
pub trait AttributePersistence: Send + Sync {
    fn save(&self, attr: &AttributeDefinition) -> Result<(), PersistenceError>;
    fn delete(&self, id: &AttributeId) -> Result<(), PersistenceError>;
    fn get(&self, id: &AttributeId) -> Result<Option<AttributeDefinition>, PersistenceError>;
    fn list_all(&self) -> Result<Vec<AttributeDefinition>, PersistenceError>;
    fn exists_by_name(&self, name: &str) -> Result<bool, PersistenceError>;
}

// src/persistence/file_storage.rs
pub struct FileAttributePersistence {
    file_path: PathBuf,
}

impl AttributePersistence for FileAttributePersistence {
    fn save(&self, attr: &AttributeDefinition) -> Result<(), PersistenceError> {
        // Load all, upsert, save with backup
        let mut all = self.list_all()?;
        
        // Find and update or insert
        if let Some(existing) = all.iter_mut().find(|a| a.id == attr.id) {
            *existing = attr.clone();
        } else {
            all.push(attr.clone());
        }
        
        // Atomic save with backup
        self.save_all(&all)?;
        Ok(())
    }
    
    // ... other implementations
}
```

#### 1.3 Add API Endpoints for Definitions (2 hours)
**File**: `src/api/routes.rs` (extend existing)

**New endpoints**:
```rust
// Attribute CRUD
POST   /api/definitions/attributes           // Create
GET    /api/definitions/attributes           // List all (existing)
GET    /api/definitions/attributes/:id       // Get one
PUT    /api/definitions/attributes/:id       // Update
DELETE /api/definitions/attributes/:id       // Delete

// Affinity CRUD
POST   /api/definitions/affinities           // Create
GET    /api/definitions/affinities           // List all (existing)
GET    /api/definitions/affinities/:id       // Get one
PUT    /api/definitions/affinities/:id       // Update
DELETE /api/definitions/affinities/:id       // Delete

// Effect CRUD
POST   /api/definitions/effects              // Create
GET    /api/definitions/effects              // List all (existing)
GET    /api/definitions/effects/:id          // Get one
PUT    /api/definitions/effects/:id          // Update
DELETE /api/definitions/effects/:id          // Delete

// Search/filter endpoints
GET    /api/definitions/attributes?q=search  // Search attributes
GET    /api/definitions/attributes?category=Physical  // Filter
```

**Handler example** (thin wrapper with NO business logic):
```rust
// src/api/handlers/definitions.rs

/// API handlers are THIN WRAPPERS ONLY
/// NO business logic in handlers - only parameter extraction and response formatting

pub async fn create_attribute(
    State(state): State<AppState>,
    Json(req): Json<CreateAttributeRequest>,
) -> Result<Json<AttributeDefinition>, ApiError> {
    // 1. Extract parameters
    let attr = AttributeDefinition {
        id: AttributeId::new(),
        name: req.name,
        description: req.description,
        category: req.category,
        base_value: req.base_value,
        min_value: req.min_value,
        max_value: req.max_value,
        training_difficulty: req.training_difficulty,
        icon: req.icon,
    };
    
    // 2. Call service (ALL business logic here)
    let created = state.attribute_service
        .create_attribute(attr)
        .map_err(|e| ApiError::from_business_error(e))?;
    
    // 3. Return response
    Ok(Json(created))
}

pub async fn update_attribute(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateAttributeRequest>,
) -> Result<Json<AttributeDefinition>, ApiError> {
    // 1. Parse ID
    let id = AttributeId::parse(&id)
        .map_err(|_| ApiError::InvalidId)?;
    
    // 2. Build attribute
    let attr = AttributeDefinition {
        id,
        name: req.name,
        description: req.description,
        category: req.category,
        base_value: req.base_value,
        min_value: req.min_value,
        max_value: req.max_value,
        training_difficulty: req.training_difficulty,
        icon: req.icon,
    };
    
    // 3. Call service (ALL business logic here)
    let updated = state.attribute_service
        .update_attribute(id, attr)
        .map_err(|e| ApiError::from_business_error(e))?;
    
    // 4. Return response
    Ok(Json(updated))
}

pub async fn delete_attribute(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    // 1. Parse ID
    let id = AttributeId::parse(&id)
        .map_err(|_| ApiError::InvalidId)?;
    
    // 2. Call service (ALL business logic here)
    state.attribute_service
        .delete_attribute(id)
        .map_err(|e| ApiError::from_business_error(e))?;
    
    // 3. Return response
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_attribute(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AttributeDefinition>, ApiError> {
    // 1. Parse ID
    let id = AttributeId::parse(&id)
        .map_err(|_| ApiError::InvalidId)?;
    
    // 2. Call service (ALL business logic here)
    let attr = state.attribute_service
        .get_attribute(id)
        .map_err(|e| ApiError::from_business_error(e))?;
    
    // 3. Return response
    Ok(Json(attr))
}

pub async fn list_attributes(
    State(state): State<AppState>,
    Query(params): Query<ListAttributesQuery>,
) -> Result<Json<Vec<AttributeDefinition>>, ApiError> {
    // 1. Call service (ALL business logic here)
    let attrs = if let Some(query) = params.q {
        state.attribute_service.search(&query)
    } else {
        state.attribute_service.list_all()
    }
    .map_err(|e| ApiError::from_business_error(e))?;
    
    // 2. Return response
    Ok(Json(attrs))
}

// Error conversion (NO business logic)
impl ApiError {
    fn from_business_error(err: BusinessError) -> Self {
        match err {
            BusinessError::NotFound => ApiError::NotFound,
            BusinessError::DuplicateName => ApiError::Conflict("Duplicate name".to_string()),
            BusinessError::ValidationError(msg) => ApiError::BadRequest(msg),
            BusinessError::IdMismatch => ApiError::BadRequest("ID mismatch".to_string()),
            BusinessError::PersistenceError(e) => ApiError::InternalServerError(e.to_string()),
        }
    }
}
```

### Phase 2: Editor Refactoring (3 hours)

#### 2.1 Remove Direct File System Access
**Files to modify**:
- `src/editor/io.rs` - **DELETE** (move logic to server)
- `src/editor/state.rs` - Remove file path dependencies
- `src/bin/editor.rs` - Load via API instead of filesystem

#### 2.2 Add API Client to Editor
**File**: `src/editor/api_client.rs` (new)

```rust
pub struct EditorApiClient {
    base_url: String,
    client: reqwest::Client,
}

impl EditorApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
        }
    }
    
    // Attribute operations
    pub async fn create_attribute(&self, attr: AttributeDefinition) 
        -> Result<AttributeDefinition, EditorApiError> {
        let url = format!("{}/api/definitions/attributes", self.base_url);
        let response = self.client
            .post(&url)
            .json(&attr)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(EditorApiError::ServerError(response.status().as_u16()));
        }
        
        Ok(response.json().await?)
    }
    
    pub async fn update_attribute(&self, id: AttributeId, attr: AttributeDefinition) 
        -> Result<AttributeDefinition, EditorApiError>;
    
    pub async fn delete_attribute(&self, id: AttributeId) 
        -> Result<(), EditorApiError>;
    
    pub async fn list_attributes(&self) 
        -> Result<Vec<AttributeDefinition>, EditorApiError>;
    
    pub async fn get_attribute(&self, id: AttributeId) 
        -> Result<AttributeDefinition, EditorApiError>;
}
```

#### 2.3 Update Editor State Management
**File**: `src/editor/state.rs` (modify)

```rust
#[derive(Debug, Resource)]
pub struct EditorState {
    /// All loaded attributes (from server)
    pub attributes: Vec<AttributeDefinition>,
    
    /// Currently selected attribute
    pub selected_index: Option<usize>,
    
    /// Attribute being edited (local copy)
    pub editing_attribute: Option<AttributeDefinition>,
    
    /// API client for server communication
    pub api_client: EditorApiClient,
    
    /// Whether current edit has unsaved changes
    pub is_dirty: bool,
    
    /// Search/filter text
    pub search_text: String,
    
    /// Status message
    pub status_message: String,
    
    /// Validation errors (from server)
    pub validation_errors: Vec<String>,
    
    /// Validation warnings (from server)
    pub validation_warnings: Vec<String>,
    
    /// Connection status
    pub server_connected: bool,
}

impl EditorState {
    pub fn new(api_url: &str) -> Self {
        Self {
            attributes: Vec::new(),
            selected_index: None,
            editing_attribute: None,
            api_client: EditorApiClient::new(api_url),
            is_dirty: false,
            search_text: String::new(),
            status_message: "Connecting to server...".to_string(),
            validation_errors: Vec::new(),
            validation_warnings: Vec::new(),
            server_connected: false,
        }
    }
    
    /// Load attributes from server (replaces load_attributes from file)
    pub async fn refresh_from_server(&mut self) -> Result<(), String> {
        match self.api_client.list_attributes().await {
            Ok(attrs) => {
                self.attributes = attrs;
                self.server_connected = true;
                self.status_message = format!("Loaded {} attributes", self.attributes.len());
                Ok(())
            }
            Err(e) => {
                self.server_connected = false;
                self.status_message = format!("Server error: {}", e);
                Err(e.to_string())
            }
        }
    }
    
    /// Save current edit to server (replaces save_to_file)
    pub async fn save_current_to_server(&mut self) -> Result<(), String> {
        if let Some(attr) = &self.editing_attribute {
            let result = if self.selected_index.is_some() {
                // Update existing
                self.api_client.update_attribute(attr.id, attr.clone()).await
            } else {
                // Create new
                self.api_client.create_attribute(attr.clone()).await
            };
            
            match result {
                Ok(saved_attr) => {
                    // Update local list
                    if let Some(idx) = self.selected_index {
                        self.attributes[idx] = saved_attr;
                    } else {
                        self.attributes.push(saved_attr);
                        self.selected_index = Some(self.attributes.len() - 1);
                    }
                    self.is_dirty = false;
                    self.status_message = "Saved successfully".to_string();
                    Ok(())
                }
                Err(e) => {
                    self.status_message = format!("Save failed: {}", e);
                    Err(e.to_string())
                }
            }
        } else {
            Err("No attribute to save".to_string())
        }
    }
    
    /// Delete selected attribute from server
    pub async fn delete_selected_from_server(&mut self) -> Result<(), String> {
        if let Some(idx) = self.selected_index {
            let attr = &self.attributes[idx];
            
            match self.api_client.delete_attribute(attr.id).await {
                Ok(_) => {
                    self.attributes.remove(idx);
                    self.selected_index = None;
                    self.editing_attribute = None;
                    self.status_message = "Deleted successfully".to_string();
                    Ok(())
                }
                Err(e) => {
                    self.status_message = format!("Delete failed: {}", e);
                    Err(e.to_string())
                }
            }
        } else {
            Err("No attribute selected".to_string())
        }
    }
}
```

#### 2.4 Update Editor UI for Async Operations
**File**: `src/editor/ui.rs` (modify)

Key changes:
- Remove "Save All" button (each edit saves individually)
- Add loading states for async operations
- Show server connection status
- Handle server errors gracefully

```rust
// Instead of save_all, use save_current for each operation
if ui.button("✓ Save").clicked() {
    should_save = true;
}

// In async handler
if should_save {
    let state_clone = state.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            state_clone.save_current_to_server().await
        })
    });
}
```

### Phase 3: Game Client Refactoring (2 hours)

#### 3.1 Extend ApiClient with Definition Mutations
**File**: `src/client/api.rs` (extend existing)

```rust
impl ApiClient {
    // ... existing methods ...
    
    // Add CRUD methods matching editor's needs
    pub async fn create_attribute(&self, attr: AttributeDefinition) 
        -> Result<AttributeDefinition, ApiError> {
        // Same as editor's API client
    }
    
    pub async fn update_attribute(&self, id: AttributeId, attr: AttributeDefinition) 
        -> Result<AttributeDefinition, ApiError> {
        // Same as editor's API client
    }
    
    pub async fn delete_attribute(&self, id: AttributeId) 
        -> Result<(), ApiError> {
        // Same as editor's API client
    }
}
```

#### 3.2 Remove Direct Definition Loading
**File**: `src/client/systems.rs` (modify)

```rust
// OLD: Load definitions on startup with blocking file I/O
pub fn load_attribute_definitions(
    api_client: Res<ApiClientResource>,
    mut definitions: ResMut<AttributeDefinitions>,
) {
    // Load from API instead of files
}

// NEW: Load definitions from server
pub fn load_attribute_definitions(
    api_client: Res<ApiClientResource>,
    mut definitions: ResMut<AttributeDefinitions>,
) {
    info!("Loading attribute definitions from server...");
    
    let client = api_client.client.clone();
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    match rt.block_on(async { client.list_attribute_definitions().await }) {
        Ok(defs) => {
            info!("✓ Loaded {} attribute definitions from server", defs.len());
            definitions.definitions = defs;
            definitions.loaded = true;
        }
        Err(e) => {
            error!("✗ Failed to load attribute definitions from server: {:?}", e);
        }
    }
}
```

### Phase 4: Testing Strategy (2 hours)

#### 4.1 Unit Tests for Business Logic (Injectable Services)
**File**: `tests/business_logic_tests.rs` (new)

**Key principle**: Test business logic independently with NO API server

```rust
#[cfg(test)]
mod attribute_service_tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;
    
    // Mock persistence layer (injectable)
    mock! {
        pub AttributePersistence {}
        
        impl AttributePersistence for AttributePersistence {
            fn save(&self, attr: &AttributeDefinition) -> Result<(), PersistenceError>;
            fn delete(&self, id: &AttributeId) -> Result<(), PersistenceError>;
            fn get(&self, id: &AttributeId) -> Result<Option<AttributeDefinition>, PersistenceError>;
            fn list_all(&self) -> Result<Vec<AttributeDefinition>, PersistenceError>;
            fn exists_by_name(&self, name: &str) -> Result<bool, PersistenceError>;
        }
    }
    
    // Mock validator (injectable)
    mock! {
        pub AttributeValidator {}
        
        impl AttributeValidator for AttributeValidator {
            fn validate(&self, attr: &AttributeDefinition, all: &[AttributeDefinition]) 
                -> Result<(), ValidationError>;
        }
    }
    
    #[test]
    fn test_create_attribute_validates() {
        // Arrange
        let mut mock_persistence = MockAttributePersistence::new();
        let mut mock_validator = MockAttributeValidator::new();
        
        mock_persistence
            .expect_list_all()
            .returning(|| Ok(vec![]));
        
        mock_validator
            .expect_validate()
            .returning(|_, _| Err(ValidationError::EmptyName));
        
        let service = AttributeServiceImpl::new(
            Arc::new(mock_persistence),
            Arc::new(mock_validator),
        );
        
        // Act
        let mut attr = AttributeDefinition::default_new();
        attr.name = "".to_string();
        let result = service.create_attribute(attr);
        
        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BusinessError::ValidationError(_)));
    }
    
    #[test]
    fn test_create_attribute_checks_duplicates() {
        // Arrange
        let mut mock_persistence = MockAttributePersistence::new();
        let mut mock_validator = MockAttributeValidator::new();
        
        mock_persistence
            .expect_list_all()
            .returning(|| Ok(vec![]));
        
        mock_persistence
            .expect_exists_by_name()
            .with(eq("Strength"))
            .returning(|_| Ok(true));
        
        mock_validator
            .expect_validate()
            .returning(|_, _| Ok(()));
        
        let service = AttributeServiceImpl::new(
            Arc::new(mock_persistence),
            Arc::new(mock_validator),
        );
        
        // Act
        let attr = AttributeDefinition {
            name: "Strength".to_string(),
            ..Default::default()
        };
        let result = service.create_attribute(attr);
        
        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BusinessError::DuplicateName));
    }
    
    #[test]
    fn test_update_attribute_success() {
        // Arrange
        let mut mock_persistence = MockAttributePersistence::new();
        let mut mock_validator = MockAttributeValidator::new();
        
        let id = AttributeId::new();
        let existing = AttributeDefinition {
            id,
            name: "Old Name".to_string(),
            ..Default::default()
        };
        
        mock_persistence
            .expect_get()
            .with(eq(id))
            .returning(move |_| Ok(Some(existing.clone())));
        
        mock_persistence
            .expect_list_all()
            .returning(|| Ok(vec![]));
        
        mock_validator
            .expect_validate()
            .returning(|_, _| Ok(()));
        
        mock_persistence
            .expect_save()
            .returning(|_| Ok(()));
        
        let service = AttributeServiceImpl::new(
            Arc::new(mock_persistence),
            Arc::new(mock_validator),
        );
        
        // Act
        let updated = AttributeDefinition {
            id,
            name: "New Name".to_string(),
            ..Default::default()
        };
        let result = service.update_attribute(id, updated);
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "New Name");
    }
    
    #[test]
    fn test_delete_attribute_not_found() {
        // Arrange
        let mut mock_persistence = MockAttributePersistence::new();
        let mock_validator = MockAttributeValidator::new();
        
        let id = AttributeId::new();
        
        mock_persistence
            .expect_get()
            .with(eq(id))
            .returning(|_| Ok(None));
        
        let service = AttributeServiceImpl::new(
            Arc::new(mock_persistence),
            Arc::new(mock_validator),
        );
        
        // Act
        let result = service.delete_attribute(id);
        
        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BusinessError::NotFound));
    }
    
    #[test]
    fn test_search_filters_by_name() {
        // Arrange
        let mut mock_persistence = MockAttributePersistence::new();
        let mock_validator = MockAttributeValidator::new();
        
        mock_persistence
            .expect_list_all()
            .returning(|| Ok(vec![
                AttributeDefinition {
                    name: "Physical Strength".to_string(),
                    ..Default::default()
                },
                AttributeDefinition {
                    name: "Mental Power".to_string(),
                    ..Default::default()
                },
                AttributeDefinition {
                    name: "Physical Endurance".to_string(),
                    ..Default::default()
                },
            ]));
        
        let service = AttributeServiceImpl::new(
            Arc::new(mock_persistence),
            Arc::new(mock_validator),
        );
        
        // Act
        let result = service.search("physical");
        
        // Assert
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|a| a.name.to_lowercase().contains("physical")));
    }
}

// Validator tests (separate from service)
#[cfg(test)]
mod attribute_validator_tests {
    use super::*;
    
    #[test]
    fn test_validate_empty_name() {
        let validator = AttributeValidatorImpl::new();
        let mut attr = AttributeDefinition::default_new();
        attr.name = "".to_string();
        
        let result = validator.validate(&attr, &[]);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_min_max_range() {
        let validator = AttributeValidatorImpl::new();
        let mut attr = AttributeDefinition::default_new();
        attr.min_value = 100;
        attr.max_value = 50;
        
        let result = validator.validate(&attr, &[]);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_duplicate_name() {
        let validator = AttributeValidatorImpl::new();
        
        let existing = AttributeDefinition {
            id: AttributeId::new(),
            name: "Strength".to_string(),
            ..Default::default()
        };
        
        let new_attr = AttributeDefinition {
            id: AttributeId::new(),
            name: "Strength".to_string(),
            ..Default::default()
        };
        
        let result = validator.validate(&new_attr, &[existing]);
        
        assert!(result.is_err());
    }
}
```

#### 4.2 Integration Tests for API
**File**: `tests/api_integration_tests.rs` (new)

```rust
#[tokio::test]
async fn test_create_attribute_endpoint() {
    let app = create_test_app().await;
    
    let req = CreateAttributeRequest {
        name: "Test Attribute".to_string(),
        description: "Test description".to_string(),
        category: AttributeCategory::Physical,
        base_value: 10,
        min_value: 1,
        max_value: 100,
        training_difficulty: Percentage::new(100),
        icon: None,
    };
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/definitions/attributes")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&req).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let created: AttributeDefinition = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(created.name, "Test Attribute");
}

#[tokio::test]
async fn test_update_attribute_endpoint() {
    // ... test update
}

#[tokio::test]
async fn test_delete_attribute_endpoint() {
    // ... test delete
}

#[tokio::test]
async fn test_validation_errors_returned() {
    let app = create_test_app().await;
    
    let req = CreateAttributeRequest {
        name: "".to_string(), // Invalid
        ..Default::default()
    };
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/definitions/attributes")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&req).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let error: ApiErrorResponse = serde_json::from_slice(&body).unwrap();
    
    assert!(error.message.contains("Name cannot be empty"));
}
```

### Phase 5: Documentation & Polish (1 hour)

#### 5.1 API Documentation
- Document all endpoints with examples
- Show request/response formats
- Document error codes
- Add architecture diagram

#### 5.2 Code Documentation
- Document all traits and their purpose
- Document dependency injection pattern
- Add examples of testing with mocks
- Document how to add new definition types

#### 5.3 Developer Guide
- How to run tests
- How to add new business logic
- How to add new API endpoints
- How to mock dependencies

**No backward compatibility layer needed** - this is a clean break from old architecture

## Implementation Timeline

### Sprint 1: Server Foundation (Week 1)
- **Day 1-2**: Create business logic layer structure
- **Day 3**: Create persistence traits and file implementations
- **Day 4-5**: Add API endpoints for attributes CRUD

**Deliverable**: Server with working CRUD API for attributes

### Sprint 2: Editor Refactor (Week 2)
- **Day 1**: Remove file system access, add API client
- **Day 2**: Update editor state management for async
- **Day 3**: Update editor UI for server communication
- **Day 4**: Testing and bug fixes

**Deliverable**: Editor working through API

### Sprint 3: Client Refactor (Week 3)
- **Day 1**: Extend game client API with mutations
- **Day 2**: Remove direct file loading
- **Day 3**: Add error handling and retry logic
- **Day 4**: Testing and integration

**Deliverable**: Game client loading definitions from API

### Sprint 4: Testing & Documentation (Week 4)
- **Day 1-2**: Unit tests for business logic with mocks
- **Day 3**: Integration tests for API (thin wrapper validation)
- **Day 4**: End-to-end testing
- **Day 5**: Documentation (architecture, testing, adding features)

**Deliverable**: Fully tested production-ready architecture with comprehensive documentation

## Benefits of New Architecture

### Testability
- ✅ Business logic can be unit tested with mocks (NO API server needed)
- ✅ API handlers are thin wrappers (minimal testing needed)
- ✅ Validation logic is injectable and independently testable
- ✅ Can test entire business layer without HTTP or file system

### Maintainability
- ✅ Clear separation of concerns (API → Service → Persistence)
- ✅ Single source of truth (server)
- ✅ Consistent patterns across all clients
- ✅ Pure Rust stack (no context switching)
- ✅ API handlers have ZERO business logic

### Scalability
- ✅ Can add web client easily (same Rust API)
- ✅ Can add mobile client in future
- ✅ Multiple editors can work simultaneously
- ✅ Can swap persistence layer (file → database)
- ✅ All components are trait-based and injectable

### Developer Experience
- ✅ Editor and game client share API client code
- ✅ Easy to add new definition types (follow same pattern)
- ✅ Clear error messages from server
- ✅ Mock all dependencies for testing
- ✅ No backward compatibility burden

### Code Quality
- ✅ Dependency injection throughout
- ✅ Interface segregation (small focused traits)
- ✅ Single responsibility (handlers only handle HTTP)
- ✅ Open/closed (extend via new implementations)
- ✅ Inversion of control (depend on abstractions)

## Migration Path

### Breaking Changes (Clean Slate)
1. **No backward compatibility** - old file-based editor stops working immediately
2. Editor must be updated to use API before it can be used
3. Game client must be updated to load from API
4. Old `editor::io` module will be deleted entirely

### Migration Steps
1. Implement server with business logic layer
2. Update editor to use API (breaks old editor)
3. Update game client to use API (breaks old client)
4. Delete old file I/O code from editor
5. Verify all tests pass

### For Existing Data
- Server loads existing JSON files on startup through persistence layer
- Files continue to be stored as JSON (format unchanged)
- Persistence layer handles all file operations
- Can migrate to database later by swapping persistence implementation

### For Developers
1. Learn dependency injection pattern
2. Write business logic in services (trait-based)
3. Keep API handlers thin (NO business logic)
4. Mock dependencies in tests
5. Test business logic independently

### For Users
- No visible changes (UI stays the same)
- Better error messages from server validation
- Multi-client support (multiple editors can run)
- More reliable (no file corruption from concurrent edits)

## Success Criteria

- ✅ Editor saves/loads through API only (file I/O deleted)
- ✅ Game client loads definitions through API only
- ✅ All validation happens in injectable services
- ✅ Business logic layer has >80% test coverage (with mocks)
- ✅ API endpoints have integration tests
- ✅ API handlers contain ZERO business logic
- ✅ All services are trait-based and injectable
- ✅ Multiple editors can connect simultaneously
- ✅ Can run tests without HTTP server
- ✅ Can run tests without file system
- ✅ Pure Rust stack throughout
- ✅ No backward compatibility code

## Future Enhancements (Post-Refactor)

### Authentication & Authorization
- User accounts
- Role-based access (admin, designer, player)
- API keys for programmatic access

### Real-time Collaboration
- WebSocket support for live updates
- Show who's editing what
- Conflict resolution

### Advanced Persistence
- Database backend (PostgreSQL, SQLite)
- Change history/audit log
- Rollback capability

### API Versioning
- `/api/v1/` prefix
- Backward compatibility guarantees
- Deprecation warnings

### Performance Optimizations
- Caching layer
- Pagination for large lists
- Bulk operations endpoint

## Notes

### Architecture Principles
- **Dependency Injection**: All business logic uses trait-based injection
- **Pure Rust**: No web technologies, JavaScript, or mixed stacks
- **No Backward Compatibility**: Clean slate until explicitly required
- **API as Thin Wrapper**: Zero business logic in HTTP handlers
- **Testability First**: All components mockable and testable independently

### Implementation Notes
- Start with attributes only (simplest case)
- Clone pattern for affinities and effects
- Delete old code after verification (no keeping "just in case")
- All traits use `Arc<dyn Trait>` for thread-safety
- Use `mockall` crate for generating mocks
- Use `#[cfg(test)]` for test-only implementations

### Testing Strategy
- Unit test business logic with mocks (NO server)
- Integration test API with real services (minimal)
- E2E test full stack (smoke tests only)
- Aim for 80%+ coverage on business logic
- Handlers don't need extensive testing (they're thin wrappers)

### Code Organization
```
src/
├── api/              # Thin HTTP wrappers ONLY
├── business/         # All business logic (injectable)
├── persistence/      # Data access traits + implementations
├── models/           # Shared data structures
├── client/           # Game client (Bevy + egui)
└── editor/           # Editor client (Bevy + egui)
```
