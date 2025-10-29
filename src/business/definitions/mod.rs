// Definition management - CRUD operations for game definitions
// All services are trait-based for dependency injection

pub mod attributes;

// Re-export traits
pub use attributes::{AttributeService, AttributeServiceImpl};

// Placeholder for future services
pub trait AffinityService: Send + Sync {
    // TODO: Implement in future phase
}

pub trait EffectService: Send + Sync {
    // TODO: Implement in future phase
}
