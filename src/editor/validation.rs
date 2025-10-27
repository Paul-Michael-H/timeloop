// Validation system for attributes

use crate::models::definitions::AttributeDefinition;
use crate::editor::EditorState;

pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
    
    pub fn add_error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
    }
    
    pub fn add_warning(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }
}

/// Validate an attribute definition
pub fn validate_attribute(attr: &AttributeDefinition, all_attributes: &[AttributeDefinition]) -> ValidationResult {
    let mut result = ValidationResult::new();
    
    // Name validation
    if attr.name.trim().is_empty() {
        result.add_error("Name cannot be empty");
    }
    
    // Check for duplicate names (excluding self)
    for other in all_attributes {
        if other.id != attr.id && other.name == attr.name {
            result.add_error(format!("Duplicate name: '{}'", attr.name));
            break;
        }
    }
    
    // Description validation
    if attr.description.trim().is_empty() {
        result.add_error("Description cannot be empty");
    }
    
    // Value range validation
    if attr.min_value >= attr.max_value {
        result.add_error(format!("Min value ({}) must be less than max value ({})", 
            attr.min_value, attr.max_value));
    }
    
    if attr.base_value < attr.min_value {
        result.add_error(format!("Base value ({}) is below min value ({})", 
            attr.base_value, attr.min_value));
    }
    
    if attr.base_value > attr.max_value {
        result.add_error(format!("Base value ({}) is above max value ({})", 
            attr.base_value, attr.max_value));
    }
    
    // Training difficulty validation
    if attr.training_difficulty.get() == 0 {
        result.add_error("Training difficulty must be greater than 0");
    }
    
    if attr.training_difficulty.get() > 1000 {
        result.add_warning(format!("Training difficulty is very high ({}%)", attr.training_difficulty.get()));
    }
    
    result
}

/// Update validation in editor state
pub fn update_validation(state: &mut EditorState) {
    if let Some(attr) = &state.editing_attribute {
        let result = validate_attribute(attr, &state.attributes);
        state.validation_errors = result.errors;
        state.validation_warnings = result.warnings;
    } else {
        state.validation_errors.clear();
        state.validation_warnings.clear();
    }
}
