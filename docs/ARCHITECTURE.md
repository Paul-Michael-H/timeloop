# Timeloop Architecture

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         CLIENT LAYER                                 │
├─────────────────────────────┬───────────────────────────────────────┤
│                             │                                        │
│    Game Client              │         Editor Client                 │
│    (Bevy + egui)            │         (Bevy + egui)                 │
│                             │                                        │
│  - Game UI                  │      - Definition Editor UI           │
│  - Character Management     │      - Validation UI                  │
│  - Progression Display      │      - CRUD Operations                │
│                             │                                        │
└─────────────┬───────────────┴───────────────┬───────────────────────┘
              │                               │
              │   HTTP REST API               │   HTTP REST API
              │   (JSON)                      │   (JSON)
              │                               │
              └───────────────┬───────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────────────┐
│                         API LAYER                                    │
│                      (Axum + Tokio)                                  │
├──────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  Routes (src/api/routes.rs)                                          │
│  ├─ /health                     - Health check                       │
│  ├─ /api/definitions/...        - CRUD for definitions              │
│  ├─ /api/game/...               - Game session management           │
│  └─ Error handling & HTTP status codes                              │
│                                                                       │
│  Handlers (src/api/handlers.rs) - THIN WRAPPERS ONLY               │
│  ├─ Request validation                                               │
│  ├─ Call business services                                           │
│  └─ Response formatting                                              │
│                                                                       │
│  AppState - Dependency Injection Container                           │
│  ├─ game_state: Arc<RwLock<GameState>>                             │
│  ├─ definitions: Arc<GameDefinitionsLoader>                          │
│  ├─ save_manager: Arc<SaveManager>                                  │
│  └─ attribute_service: Arc<AttributeServiceImpl>                    │
│                                                                       │
└───────────────────────────────┬──────────────────────────────────────┘
                                │
┌───────────────────────────────▼──────────────────────────────────────┐
│                      BUSINESS LOGIC LAYER                            │
│                   (src/business/) - CORE LOGIC                       │
├──────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  Definitions (src/business/definitions/)                             │
│  ├─ AttributeService trait - Interface for DI                       │
│  ├─ AttributeServiceImpl - Implementation                            │
│  ├─ AffinityService (future)                                         │
│  └─ EffectService (future)                                           │
│                                                                       │
│  Validation (src/business/validation/)                               │
│  ├─ AttributeValidator trait - Interface for DI                     │
│  ├─ AttributeValidatorImpl - Rules engine                           │
│  ├─ Validates data integrity                                         │
│  └─ NO persistence knowledge                                         │
│                                                                       │
│  Game State (src/business/game_state/)                              │
│  ├─ Character progression logic                                      │
│  ├─ Training calculations                                            │
│  └─ Game rules enforcement                                           │
│                                                                       │
│  ⚠️  CRITICAL: NO DIRECT STORAGE ACCESS                             │
│     All persistence via injected traits                              │
│                                                                       │
└───────────────────────────────┬──────────────────────────────────────┘
                                │
┌───────────────────────────────▼──────────────────────────────────────┐
│                     PERSISTENCE LAYER                                │
│              (src/persistence/) - STORAGE ABSTRACTION                │
├──────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  Traits (src/persistence/traits.rs)                                 │
│  ├─ AttributePersistence trait - Abstract storage interface         │
│  ├─ save(attr) -> Result<()>                                        │
│  ├─ get(id) -> Result<Option<Attribute>>                            │
│  ├─ delete(id) -> Result<()>                                        │
│  ├─ list_all() -> Result<Vec<Attribute>>                            │
│  ├─ exists_by_name(name) -> Result<bool>                            │
│  └─ search_by_name(query) -> Result<Vec<Attribute>>                 │
│                                                                       │
│  Implementations:                                                     │
│  ├─ FileAttributePersistence (src/persistence/file_storage.rs)      │
│  │   ├─ JSON file storage                                            │
│  │   ├─ Caching for performance                                      │
│  │   └─ Atomic writes                                                │
│  │                                                                    │
│  └─ InMemoryAttributePersistence (src/persistence/memory_storage.rs)│
│      ├─ HashMap-based storage                                        │
│      ├─ For testing only                                             │
│      └─ No file I/O                                                  │
│                                                                       │
└───────────────────────────────┬──────────────────────────────────────┘
                                │
┌───────────────────────────────▼──────────────────────────────────────┐
│                       STORAGE LAYER                                  │
│                    (File System / Database)                          │
├──────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  File System (Current)                                               │
│  ├─ game_data/core/attributes.json                                  │
│  ├─ game_data/core/affinities.json                                  │
│  ├─ game_data/core/effects.json                                     │
│  └─ saves/*.json                                                     │
│                                                                       │
│  Database (Future)                                                   │
│  ├─ PostgreSQL                                                       │
│  ├─ SQLite                                                           │
│  └─ Any storage via trait swap                                      │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

---

## Dependency Flow

```
Editor/Game Client
        ↓
    API Handlers (thin wrappers)
        ↓
    Business Services (trait-based)
        ↓
    Persistence Traits (abstract storage)
        ↓
    Storage Implementation (file/DB/memory)
```

**Key Principle**: Each layer only knows about the layer directly below it through traits.

---

## Data Flow: Creating an Attribute

```
1. Editor UI
   └─> User enters attribute data
       └─> Validate locally (optional)

2. API Client (src/editor/api_client.rs)
   └─> POST /api/definitions/attributes
       └─> JSON serialized attribute

3. API Handler (src/api/handlers.rs)
   └─> Deserialize request
   └─> Extract AppState
   └─> Call: state.attribute_service.create_attribute(attr)
   └─> Convert BusinessError → ApiError → HTTP status

4. Business Logic (src/business/definitions/attributes.rs)
   └─> Validate via AttributeValidator
   └─> Check uniqueness via persistence.exists_by_name()
   └─> Call: persistence.save(attr)
   └─> Return result

5. Persistence (src/persistence/file_storage.rs)
   └─> Load existing attributes (if not cached)
   └─> Add new attribute
   └─> Atomic write to JSON file
   └─> Update cache
   └─> Return success

6. Response flows back up:
   API Handler → HTTP 201 Created → API Client → UI Update
```

---

## Dependency Injection Pattern

### Setup (src/bin/server.rs)

```rust
// 1. Create persistence (injectable)
let persistence = Arc::new(FileAttributePersistence::new(path));

// 2. Create validator (injectable)
let validator = Arc::new(AttributeValidatorImpl::new());

// 3. Create service with injected dependencies
let attribute_service = Arc::new(
    AttributeServiceImpl::new(persistence, validator)
);

// 4. Create AppState with injected service
let state = AppState {
    attribute_service,
    // ... other fields
};

// 5. Pass state to router
let app = create_router(state);
```

### Benefits

- **Testability**: Swap real persistence with in-memory for tests
- **Flexibility**: Change storage without touching business logic
- **Maintainability**: Clear boundaries and responsibilities
- **No coupling**: Business logic has zero storage knowledge

---

## Module Structure

```
src/
├── api/                    # API Layer
│   ├── handlers.rs         # Thin HTTP handlers
│   ├── routes.rs           # Route definitions
│   ├── middleware.rs       # Future: auth, logging
│   └── mod.rs
│
├── business/               # Business Logic Layer
│   ├── definitions/        # Definition management
│   │   ├── attributes.rs   # AttributeService + impl
│   │   └── mod.rs
│   ├── validation/         # Validation rules
│   │   ├── attributes.rs   # AttributeValidator + impl
│   │   └── mod.rs
│   ├── game_state/         # Game logic
│   │   └── mod.rs
│   └── mod.rs              # BusinessError enum
│
├── persistence/            # Persistence Layer
│   ├── traits.rs           # Storage traits
│   ├── file_storage.rs     # File implementation
│   ├── memory_storage.rs   # Test implementation
│   └── mod.rs              # PersistenceError enum
│
├── editor/                 # Editor Client
│   ├── api_client.rs       # REST API client
│   ├── state.rs            # Editor state management
│   ├── ui.rs               # UI rendering
│   ├── validation.rs       # UI validation
│   └── mod.rs
│
├── client/                 # Game Client
│   ├── api.rs              # REST API client
│   ├── systems.rs          # Bevy systems
│   ├── ui.rs               # Game UI
│   └── mod.rs
│
├── models/                 # Shared Data Models
│   ├── definitions.rs      # AttributeDefinition, etc.
│   ├── instances.rs        # CharacterAttribute, etc.
│   ├── common.rs           # AttributeId, Percentage, etc.
│   └── mod.rs
│
├── storage/                # Legacy (to be refactored)
│   ├── game_data_loader.rs
│   └── save_manager.rs
│
└── bin/                    # Executables
    ├── server.rs           # API server (Axum)
    ├── editor.rs           # Editor client (Bevy + egui)
    └── [game client binary name].rs
```

---

## Testing Architecture

```
tests/
├── business_logic_tests.rs      # Unit tests with mocks
│   └─> Uses InMemoryPersistence
│   └─> Tests AttributeService logic
│   └─> Zero file I/O
│
├── api_integration_tests.rs     # API endpoint tests
│   └─> Uses test router
│   └─> Tests HTTP layer
│   └─> Verifies status codes
│
├── editor_tests.rs              # Editor state tests
│   └─> UI state management
│   └─> Validation logic
│
└── integration_test.rs          # Full stack tests
    └─> Game session flow
    └─> Save/load operations
```

### Test Coverage Targets

- Business Logic: **≥90%** (currently ~95%)
- Persistence Layer: **100%** (achieved)
- API Handlers: **Good** (integration tests)
- Editor/Client: **Good** (unit tests)

---

## Key Architectural Decisions

### 1. Trait-Based Services
**Why**: Enables dependency injection and testing
**Impact**: All services are swappable via traits

### 2. Zero Business Logic in API Handlers
**Why**: Maintainability and testability
**Impact**: Handlers are pure wrappers (~5 lines each)

### 3. Persistence Abstraction
**Why**: Storage independence
**Impact**: Can swap file → database without changing business logic

### 4. Async/Await Throughout
**Why**: Non-blocking I/O and scalability
**Impact**: All persistence and API operations are async

### 5. Pure Rust Stack
**Why**: Performance, type safety, single language
**Impact**: No JavaScript, no separate frontend/backend stacks

---

## Security Considerations

### Current Status
- ❌ No authentication
- ❌ No authorization
- ❌ No rate limiting
- ❌ No input sanitization beyond validation
- ✅ No SQL injection risk (no SQL yet)
- ✅ No XSS risk (no HTML rendering)

### Future Enhancements
1. JWT-based authentication
2. Role-based access control (RBAC)
3. Rate limiting per endpoint
4. Request signing
5. Audit logging

---

## Performance Considerations

### Current Optimizations
- ✅ Caching in FileAttributePersistence
- ✅ Atomic writes to prevent corruption
- ✅ Async I/O throughout
- ✅ Minimal allocations

### Future Optimizations
1. Connection pooling for database
2. Redis caching layer
3. GraphQL for flexible queries
4. WebSocket for real-time updates
5. Binary protocol for efficiency

---

## Scalability

### Current Limitations
- Single-server architecture
- File-based storage
- No horizontal scaling
- No load balancing

### Future Scaling Path
1. Replace file storage with PostgreSQL
2. Add Redis for session management
3. Horizontal scaling with load balancer
4. Microservices for heavy operations
5. Message queue for async tasks

---

## Development Workflow

```
1. Define trait in persistence layer
2. Implement trait (file + memory versions)
3. Create business service using trait
4. Add API handler calling service
5. Write tests (unit + integration)
6. Update clients (editor + game)
7. Document API endpoint
```

---

## Migration Path

### From Current Architecture
```
OLD: Editor → File I/O → JSON
NEW: Editor → API → Business Logic → Persistence → JSON
```

### Benefits
- ✅ Multi-client support
- ✅ Centralized validation
- ✅ Better error handling
- ✅ Easy to test
- ✅ Future-proof for database

---

## Contributing

See `docs/DEVELOPER_GUIDE.md` for:
- Setup instructions
- Coding standards
- Testing requirements
- PR process
