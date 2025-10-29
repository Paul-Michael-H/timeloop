// Persistence layer - abstracts storage mechanisms
// Business logic depends on traits defined here, NOT on implementations
// This allows swapping storage (file -> DB -> memory -> network) without changing business logic

pub mod traits;
pub mod file_storage;
pub mod memory_storage;

// Re-export main traits
pub use traits::{
    AttributePersistence,
    PersistenceError,
};

// Re-export implementations
pub use file_storage::FileAttributePersistence;
pub use memory_storage::InMemoryAttributePersistence;
