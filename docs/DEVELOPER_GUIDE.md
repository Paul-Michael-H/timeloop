# Timeloop Developer Guide

## Getting Started

### Prerequisites

- **Rust**: 1.75 or later (`rustup update`)
- **Cargo**: Included with Rust
- **Git**: For version control
- **VS Code** (recommended): With rust-analyzer extension

### Initial Setup

```powershell
# Clone repository
git clone <repository-url>
cd timeloop

# Build project
cargo build

# Run tests to verify setup
cargo test

# Run clippy to check code quality
cargo clippy -- -D warnings

# Start server
cargo run --bin timeloop-server

# In another terminal, start editor
cargo run --bin timeloop-editor
```

---

## Project Structure

See `docs/ARCHITECTURE.md` for detailed module structure. Key principles:

- **API Layer**: Thin handlers only, no business logic
- **Business Layer**: Core logic, zero storage knowledge
- **Persistence Layer**: Storage abstraction via traits
- **Models**: Shared data structures across all layers

---

## Coding Standards

### Quality Gates (MANDATORY)

Every phase and PR must pass:

✅ **Zero compiler warnings**  
✅ **Zero clippy remarks** (`cargo clippy -- -D warnings`)  
✅ **All tests passing** (`cargo test`)  
✅ **≥90% test coverage** for business logic  

### Code Style

```rust
// ✅ Good: Trait-based dependency injection
pub trait AttributePersistence: Send + Sync {
    async fn save(&self, attr: AttributeDefinition) -> Result<(), PersistenceError>;
    async fn get(&self, id: &AttributeId) -> Result<Option<AttributeDefinition>, PersistenceError>;
}

pub struct AttributeServiceImpl<P: AttributePersistence> {
    persistence: Arc<P>,
    validator: Arc<dyn AttributeValidator>,
}

// ❌ Bad: Direct storage access
pub struct AttributeService {
    file_path: PathBuf,  // Business logic shouldn't know about storage!
}
```

```rust
// ✅ Good: Async throughout
pub async fn create_attribute(&self, attr: AttributeDefinition) -> Result<AttributeDefinition, BusinessError> {
    self.validator.validate(&attr)?;
    self.persistence.save(attr.clone()).await?;
    Ok(attr)
}

// ❌ Bad: Blocking I/O
pub fn create_attribute(&self, attr: AttributeDefinition) -> Result<AttributeDefinition, BusinessError> {
    std::fs::write("file.json", data)?;  // Blocks entire async runtime!
}
```

```rust
// ✅ Good: Thin API handlers
pub async fn create_attribute_handler(
    State(state): State<AppState>,
    Json(attr): Json<AttributeDefinition>,
) -> Result<Json<AttributeDefinition>, ApiError> {
    let result = state.attribute_service.create_attribute(attr).await?;
    Ok(Json(result))
}

// ❌ Bad: Business logic in handler
pub async fn create_attribute_handler(...) -> Result<...> {
    // Validation logic here
    // Duplicate checking here
    // File writing here
    // This should all be in business layer!
}
```

### Naming Conventions

- **Types**: `PascalCase` (e.g., `AttributeDefinition`)
- **Functions**: `snake_case` (e.g., `create_attribute`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_LEVEL`)
- **Traits**: Descriptive nouns ending in trait purpose (e.g., `AttributePersistence`, `AttributeValidator`)
- **Implementations**: Trait name + `Impl` (e.g., `AttributeServiceImpl`, `AttributeValidatorImpl`)

### Error Handling

```rust
// ✅ Good: Layer-specific errors
#[derive(Debug, thiserror::Error)]
pub enum BusinessError {
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Duplicate name: {0}")]
    DuplicateName(String),
    
    #[error("Persistence error: {0}")]
    PersistenceError(#[from] PersistenceError),
}

// Convert at layer boundaries
impl From<BusinessError> for ApiError {
    fn from(err: BusinessError) -> Self {
        match err {
            BusinessError::DuplicateName(msg) => ApiError::Conflict(msg),
            BusinessError::ValidationFailed(msg) => ApiError::BadRequest(msg),
            _ => ApiError::InternalError(err.to_string()),
        }
    }
}
```

### Documentation

```rust
/// Creates a new attribute definition.
///
/// # Arguments
/// * `attr` - The attribute definition to create
///
/// # Returns
/// * `Ok(AttributeDefinition)` - The created attribute
/// * `Err(BusinessError::ValidationFailed)` - If validation fails
/// * `Err(BusinessError::DuplicateName)` - If name already exists
///
/// # Examples
/// ```
/// let attr = AttributeDefinition::new("Strength", ...);
/// let result = service.create_attribute(attr).await?;
/// ```
pub async fn create_attribute(&self, attr: AttributeDefinition) -> Result<AttributeDefinition, BusinessError> {
    // Implementation
}
```

---

## Adding a New Feature

### Example: Adding AffinityService

#### Step 1: Define Persistence Trait

**File**: `src/persistence/traits.rs`

```rust
#[async_trait]
pub trait AffinityPersistence: Send + Sync {
    async fn save(&self, affinity: AffinityDefinition) -> Result<(), PersistenceError>;
    async fn get(&self, id: &AffinityId) -> Result<Option<AffinityDefinition>, PersistenceError>;
    async fn delete(&self, id: &AffinityId) -> Result<(), PersistenceError>;
    async fn list_all(&self) -> Result<Vec<AffinityDefinition>, PersistenceError>;
    async fn exists_by_name(&self, name: &str) -> Result<bool, PersistenceError>;
}
```

#### Step 2: Implement Persistence

**File**: `src/persistence/file_storage.rs`

```rust
pub struct FileAffinityPersistence {
    file_path: PathBuf,
    cache: RwLock<Option<Vec<AffinityDefinition>>>,
}

#[async_trait]
impl AffinityPersistence for FileAffinityPersistence {
    async fn save(&self, affinity: AffinityDefinition) -> Result<(), PersistenceError> {
        let mut affinities = self.load_all().await?;
        
        // Update or insert
        if let Some(pos) = affinities.iter().position(|a| a.id == affinity.id) {
            affinities[pos] = affinity;
        } else {
            affinities.push(affinity);
        }
        
        self.write_all(&affinities).await?;
        Ok(())
    }
    
    // ... implement other methods
}
```

**File**: `src/persistence/memory_storage.rs` (for testing)

```rust
pub struct InMemoryAffinityPersistence {
    data: RwLock<HashMap<AffinityId, AffinityDefinition>>,
}

#[async_trait]
impl AffinityPersistence for InMemoryAffinityPersistence {
    async fn save(&self, affinity: AffinityDefinition) -> Result<(), PersistenceError> {
        let mut data = self.data.write().unwrap();
        data.insert(affinity.id.clone(), affinity);
        Ok(())
    }
    
    // ... implement other methods
}
```

#### Step 3: Create Validator

**File**: `src/business/validation/affinities.rs`

```rust
#[async_trait]
pub trait AffinityValidator: Send + Sync {
    fn validate(&self, affinity: &AffinityDefinition) -> Result<(), ValidationError>;
}

pub struct AffinityValidatorImpl;

impl AffinityValidatorImpl {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AffinityValidator for AffinityValidatorImpl {
    fn validate(&self, affinity: &AffinityDefinition) -> Result<(), ValidationError> {
        if affinity.name.trim().is_empty() {
            return Err(ValidationError::EmptyField("name".to_string()));
        }
        
        if affinity.name.len() > 50 {
            return Err(ValidationError::TooLong("name".to_string(), 50));
        }
        
        // More validation rules...
        
        Ok(())
    }
}
```

#### Step 4: Create Business Service

**File**: `src/business/definitions/affinities.rs`

```rust
#[async_trait]
pub trait AffinityService: Send + Sync {
    async fn create_affinity(&self, affinity: AffinityDefinition) -> Result<AffinityDefinition, BusinessError>;
    async fn get_affinity(&self, id: &AffinityId) -> Result<AffinityDefinition, BusinessError>;
    async fn update_affinity(&self, affinity: AffinityDefinition) -> Result<AffinityDefinition, BusinessError>;
    async fn delete_affinity(&self, id: &AffinityId) -> Result<(), BusinessError>;
    async fn list_affinities(&self) -> Result<Vec<AffinityDefinition>, BusinessError>;
}

pub struct AffinityServiceImpl<P: AffinityPersistence> {
    persistence: Arc<P>,
    validator: Arc<dyn AffinityValidator>,
}

impl<P: AffinityPersistence> AffinityServiceImpl<P> {
    pub fn new(persistence: Arc<P>, validator: Arc<dyn AffinityValidator>) -> Self {
        Self { persistence, validator }
    }
}

#[async_trait]
impl<P: AffinityPersistence> AffinityService for AffinityServiceImpl<P> {
    async fn create_affinity(&self, affinity: AffinityDefinition) -> Result<AffinityDefinition, BusinessError> {
        // Validate
        self.validator.validate(&affinity)
            .map_err(|e| BusinessError::ValidationFailed(e.to_string()))?;
        
        // Check for duplicates
        if self.persistence.exists_by_name(&affinity.name).await? {
            return Err(BusinessError::DuplicateName(affinity.name.clone()));
        }
        
        // Save
        self.persistence.save(affinity.clone()).await?;
        
        Ok(affinity)
    }
    
    // ... implement other methods
}
```

#### Step 5: Add API Handler

**File**: `src/api/handlers.rs`

```rust
pub async fn create_affinity_handler(
    State(state): State<AppState>,
    Json(affinity): Json<AffinityDefinition>,
) -> Result<Json<AffinityDefinition>, ApiError> {
    let result = state.affinity_service.create_affinity(affinity).await?;
    Ok(Json(result))
}

pub async fn get_affinity_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AffinityDefinition>, ApiError> {
    let id = AffinityId::new(id);
    let affinity = state.affinity_service.get_affinity(&id).await?;
    Ok(Json(affinity))
}

// ... other handlers
```

**File**: `src/api/routes.rs`

```rust
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        
        // Attribute routes
        .route("/api/definitions/attributes", post(create_attribute_handler))
        // ...
        
        // NEW: Affinity routes
        .route("/api/definitions/affinities", post(create_affinity_handler))
        .route("/api/definitions/affinities", get(list_affinities_handler))
        .route("/api/definitions/affinities/:id", get(get_affinity_handler))
        .route("/api/definitions/affinities/:id", put(update_affinity_handler))
        .route("/api/definitions/affinities/:id", delete(delete_affinity_handler))
        
        .with_state(state)
}
```

#### Step 6: Update AppState

**File**: `src/bin/server.rs`

```rust
#[derive(Clone)]
pub struct AppState {
    pub attribute_service: Arc<dyn AttributeService>,
    pub affinity_service: Arc<dyn AffinityService>,  // NEW
    // ... other fields
}

#[tokio::main]
async fn main() {
    // Attribute setup
    let attr_persistence = Arc::new(FileAttributePersistence::new("game_data/core/attributes.json"));
    let attr_validator = Arc::new(AttributeValidatorImpl::new());
    let attribute_service = Arc::new(AttributeServiceImpl::new(attr_persistence, attr_validator));
    
    // NEW: Affinity setup
    let aff_persistence = Arc::new(FileAffinityPersistence::new("game_data/core/affinities.json"));
    let aff_validator = Arc::new(AffinityValidatorImpl::new());
    let affinity_service = Arc::new(AffinityServiceImpl::new(aff_persistence, aff_validator));
    
    let state = AppState {
        attribute_service,
        affinity_service,  // NEW
    };
    
    let app = create_router(state);
    // ... start server
}
```

#### Step 7: Write Tests

**File**: `tests/affinity_business_tests.rs`

```rust
use timeloop::persistence::memory_storage::InMemoryAffinityPersistence;
use timeloop::business::definitions::affinities::{AffinityService, AffinityServiceImpl};
use timeloop::business::validation::affinities::AffinityValidatorImpl;

#[tokio::test]
async fn test_create_affinity_success() {
    let persistence = Arc::new(InMemoryAffinityPersistence::new());
    let validator = Arc::new(AffinityValidatorImpl::new());
    let service = AffinityServiceImpl::new(persistence, validator);
    
    let affinity = AffinityDefinition::new("Fire", "fire");
    let result = service.create_affinity(affinity).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_duplicate_affinity_fails() {
    let persistence = Arc::new(InMemoryAffinityPersistence::new());
    let validator = Arc::new(AffinityValidatorImpl::new());
    let service = AffinityServiceImpl::new(persistence, validator);
    
    let affinity = AffinityDefinition::new("Fire", "fire");
    service.create_affinity(affinity.clone()).await.unwrap();
    
    let result = service.create_affinity(affinity).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BusinessError::DuplicateName(_)));
}

// ... more tests
```

**File**: `tests/affinity_api_tests.rs`

```rust
use axum::http::StatusCode;
use tower::ServiceExt;

#[tokio::test]
async fn test_create_affinity_endpoint() {
    let state = create_test_app_state();
    let app = create_router(state);
    
    let affinity = json!({
        "id": "fire_id",
        "name": "Fire",
        "icon": "fire"
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/definitions/affinities")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&affinity).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::CREATED);
}

// ... more tests
```

#### Step 8: Run Quality Checks

```powershell
# Build
cargo build
# ✅ Should compile with zero warnings

# Tests
cargo test
# ✅ All tests should pass

# Clippy
cargo clippy -- -D warnings
# ✅ No clippy remarks

# Coverage (optional but recommended)
cargo tarpaulin --out Html
# ✅ Should show ≥90% for new business logic
```

#### Step 9: Update Documentation

**File**: `docs/API_DOCUMENTATION.md`

Add section for affinity endpoints with examples.

**File**: `docs/ARCHITECTURE.md`

Update diagrams if needed.

---

## Testing Best Practices

### Unit Tests (Business Logic)

```rust
// ✅ Good: Use in-memory persistence
#[tokio::test]
async fn test_business_logic() {
    let persistence = Arc::new(InMemoryAttributePersistence::new());
    let validator = Arc::new(AttributeValidatorImpl::new());
    let service = AttributeServiceImpl::new(persistence, validator);
    
    // Test logic, not I/O
}

// ❌ Bad: Use file persistence in unit tests
#[tokio::test]
async fn test_business_logic() {
    let persistence = Arc::new(FileAttributePersistence::new("test.json"));
    // Slow, fragile, not isolated!
}
```

### Integration Tests (API)

```rust
// ✅ Good: Test HTTP layer
#[tokio::test]
async fn test_api_endpoint() {
    let app = create_test_router();
    
    let response = app.oneshot(
        Request::builder()
            .uri("/api/definitions/attributes")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(json_string))
            .unwrap()
    ).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::CREATED);
}
```

### Test Organization

```
tests/
├── <feature>_business_tests.rs    # Unit tests with mocks
├── <feature>_api_tests.rs         # Integration tests for HTTP
└── <feature>_integration_test.rs  # Full stack tests
```

### Coverage Goals

- **Business Logic**: ≥90% coverage
- **Persistence**: 100% coverage (simple CRUD)
- **API Handlers**: Good coverage via integration tests
- **UI**: Basic coverage for state management

---

## Common Patterns

### Pattern 1: Creating a Service

```rust
// 1. Define trait
#[async_trait]
pub trait MyService: Send + Sync {
    async fn do_something(&self) -> Result<(), BusinessError>;
}

// 2. Create implementation with injected dependencies
pub struct MyServiceImpl<P: MyPersistence> {
    persistence: Arc<P>,
    validator: Arc<dyn MyValidator>,
}

// 3. Implement trait
#[async_trait]
impl<P: MyPersistence> MyService for MyServiceImpl<P> {
    async fn do_something(&self) -> Result<(), BusinessError> {
        // Use injected dependencies
        self.validator.validate()?;
        self.persistence.save().await?;
        Ok(())
    }
}
```

### Pattern 2: Error Conversion

```rust
// Define error at each layer
pub enum PersistenceError { IoError, SerializationError }
pub enum BusinessError { ValidationFailed, PersistenceError }
pub enum ApiError { BadRequest, InternalError }

// Convert at boundaries
impl From<PersistenceError> for BusinessError {
    fn from(err: PersistenceError) -> Self {
        BusinessError::PersistenceError(err)
    }
}

impl From<BusinessError> for ApiError {
    fn from(err: BusinessError) -> Self {
        match err {
            BusinessError::ValidationFailed(msg) => ApiError::BadRequest(msg),
            _ => ApiError::InternalError(err.to_string()),
        }
    }
}
```

### Pattern 3: Async Handler

```rust
pub async fn my_handler(
    State(state): State<AppState>,
    Json(payload): Json<MyType>,
) -> Result<Json<MyResponse>, ApiError> {
    // 1. Extract dependencies from state
    let service = &state.my_service;
    
    // 2. Call business logic
    let result = service.do_something(payload).await?;
    
    // 3. Return response
    Ok(Json(result))
}
```

---

## Debugging Tips

### Enable Logging

```rust
// In main function
tracing_subscriber::fmt::init();

// In code
tracing::info!("Processing request");
tracing::error!("Failed to save: {}", err);
```

Run with:
```powershell
$env:RUST_LOG="debug"; cargo run --bin timeloop-server
```

### Test a Single Test

```powershell
cargo test test_create_attribute_success -- --nocapture
```

### Check Test Coverage

```powershell
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage
```

Open `coverage/index.html` in browser.

### Debug API Requests

```powershell
# With curl
curl -X POST http://localhost:3000/api/definitions/attributes `
  -H "Content-Type: application/json" `
  -d '{"id":"str_id","name":"Strength","icon":"muscle"}'

# With PowerShell
$body = @{
    id = "str_id"
    name = "Strength"
    icon = "muscle"
} | ConvertTo-Json

Invoke-RestMethod -Uri "http://localhost:3000/api/definitions/attributes" `
  -Method Post `
  -Body $body `
  -ContentType "application/json"
```

---

## Git Workflow

### Branching Strategy

```
main                    # Production-ready code
├── feature/affinities  # Feature branches
├── bugfix/validation   # Bug fixes
└── refactor/api        # Refactoring work
```

### Commit Messages

```
feat: Add affinity service with CRUD operations

- Implement AffinityPersistence trait with file and memory versions
- Create AffinityService with validation
- Add API endpoints for affinity management
- Write 15 unit tests and 8 integration tests

Quality gates: ✅ Zero warnings ✅ Zero clippy ✅ All tests pass ✅ 95% coverage
```

### PR Checklist

- [ ] All tests passing
- [ ] Zero compiler warnings
- [ ] Zero clippy remarks
- [ ] ≥90% test coverage for new business logic
- [ ] Documentation updated (API docs, architecture if needed)
- [ ] Code reviewed by at least one other developer

---

## Performance Tips

### 1. Use Caching

```rust
pub struct FileAttributePersistence {
    cache: RwLock<Option<Vec<AttributeDefinition>>>,
}

async fn load_all(&self) -> Result<Vec<AttributeDefinition>, PersistenceError> {
    // Check cache first
    {
        let cache = self.cache.read().unwrap();
        if let Some(cached) = cache.as_ref() {
            return Ok(cached.clone());
        }
    }
    
    // Load from file
    let data = self.read_file().await?;
    
    // Update cache
    {
        let mut cache = self.cache.write().unwrap();
        *cache = Some(data.clone());
    }
    
    Ok(data)
}
```

### 2. Minimize Allocations

```rust
// ✅ Good: Reuse allocations
let mut buffer = String::with_capacity(1024);
for item in items {
    buffer.clear();
    write!(&mut buffer, "{}", item)?;
}

// ❌ Bad: Allocate on every iteration
for item in items {
    let buffer = format!("{}", item);  // New allocation each time
}
```

### 3. Batch Operations

```rust
// ✅ Good: Save multiple items at once
pub async fn save_all(&self, items: Vec<Item>) -> Result<(), Error> {
    let mut all_items = self.load_all().await?;
    all_items.extend(items);
    self.write_all(&all_items).await?;
    Ok(())
}

// ❌ Bad: Save one at a time (N file writes!)
for item in items {
    self.save(item).await?;
}
```

---

## Security Checklist

- [ ] No secrets in code (use environment variables)
- [ ] Input validation on all endpoints
- [ ] Error messages don't leak sensitive info
- [ ] File paths are sanitized (no directory traversal)
- [ ] Rate limiting on expensive operations
- [ ] Authentication for sensitive endpoints
- [ ] Authorization checks before mutations

---

## FAQ

### Q: Why use traits instead of concrete types?

**A**: Enables dependency injection and testing. You can swap file storage for in-memory storage in tests without changing business logic.

### Q: Why async everywhere?

**A**: Enables non-blocking I/O and scalability. Essential for server applications handling multiple concurrent requests.

### Q: Can I add business logic to API handlers?

**A**: No. Handlers must be thin wrappers. All logic goes in business services.

### Q: When should I create a new service?

**A**: When you have a cohesive set of operations on a specific entity (e.g., AttributeService for attributes, AffinityService for affinities).

### Q: How do I mock dependencies in tests?

**A**: Use the in-memory persistence implementations (e.g., `InMemoryAttributePersistence`) instead of file-based ones.

### Q: What if I need to access multiple services in one handler?

**A**: Create a facade service that coordinates multiple services, or inject all needed services into `AppState`.

---

## Resources

- **Rust Book**: https://doc.rust-lang.org/book/
- **Async Rust**: https://rust-lang.github.io/async-book/
- **Axum Docs**: https://docs.rs/axum/
- **Bevy Docs**: https://bevyengine.org/learn/
- **Tokio Docs**: https://tokio.rs/tokio/tutorial

---

## Contact

For questions or issues:
- Create an issue in the repository
- Tag `@<maintainer>` in Discord/Slack
- Email: `<team-email>`

---

**Remember**: Quality gates are mandatory. Every change must have zero warnings, zero clippy remarks, all tests passing, and ≥90% coverage.
