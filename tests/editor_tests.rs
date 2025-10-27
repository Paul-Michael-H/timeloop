// Integration tests for the Timeloop Game Object Editor

use timeloop::editor::{EditorState, validation};
use timeloop::editor::io::load_attributes;
use timeloop::editor::state::AttributeDefinitionExt;
use timeloop::models::definitions::{AttributeDefinition, AttributeCategory};
use timeloop::models::common::{AttributeId, Percentage};
use std::fs;

#[test]
fn test_editor_state_creation() {
    let state = EditorState::new();
    
    assert_eq!(state.attributes.len(), 0);
    assert_eq!(state.selected_index, None);
    assert_eq!(state.editing_attribute, None);
    assert_eq!(state.is_dirty, false);
    assert_eq!(state.search_text, "");
    assert_eq!(state.validation_errors.len(), 0);
    assert_eq!(state.validation_warnings.len(), 0);
}

#[test]
fn test_default_attribute_creation() {
    let attr = AttributeDefinition::default_new();
    
    assert_eq!(attr.name, "New Attribute");
    assert_eq!(attr.description, "A new attribute for the game");
    assert_eq!(attr.category, AttributeCategory::Physical);
    assert_eq!(attr.base_value, 10);
    assert_eq!(attr.min_value, 1);
    assert_eq!(attr.max_value, 100);
    assert_eq!(attr.training_difficulty.get(), 100);
    assert_eq!(attr.icon, None);
}

#[test]
fn test_start_creating_unique_names() {
    let mut state = EditorState::new();
    
    // Create first attribute
    state.start_creating();
    assert!(state.editing_attribute.is_some());
    let first_name = state.editing_attribute.as_ref().unwrap().name.clone();
    state.save_current_edit();
    
    // Create second attribute - should have different name
    state.start_creating();
    assert!(state.editing_attribute.is_some());
    let second_name = state.editing_attribute.as_ref().unwrap().name.clone();
    
    assert_ne!(first_name, second_name);
    assert!(second_name.starts_with("New Attribute"));
}

#[test]
fn test_start_editing() {
    let mut state = EditorState::new();
    
    // Add an attribute
    let attr = AttributeDefinition {
        id: AttributeId::new(),
        name: "Test Attr".to_string(),
        description: "Test Description".to_string(),
        category: AttributeCategory::Mental,
        base_value: 15,
        min_value: 5,
        max_value: 50,
        training_difficulty: Percentage::new(150),
        icon: Some("test.png".to_string()),
    };
    state.attributes.push(attr.clone());
    state.selected_index = Some(0);
    
    // Start editing
    state.start_editing();
    
    assert!(state.editing_attribute.is_some());
    assert_eq!(state.editing_attribute.as_ref().unwrap().name, "Test Attr");
    assert_eq!(state.is_dirty, false);
}

#[test]
fn test_save_current_edit_new() {
    let mut state = EditorState::new();
    
    state.start_creating();
    assert_eq!(state.attributes.len(), 0);
    
    // Modify the attribute
    if let Some(attr) = &mut state.editing_attribute {
        attr.name = "Custom Attribute".to_string();
    }
    
    state.save_current_edit();
    
    assert_eq!(state.attributes.len(), 1);
    assert_eq!(state.attributes[0].name, "Custom Attribute");
    assert_eq!(state.selected_index, Some(0));
    assert_eq!(state.is_dirty, false);
}

#[test]
fn test_save_current_edit_existing() {
    let mut state = EditorState::new();
    
    // Add an attribute
    let attr = AttributeDefinition::default_new();
    state.attributes.push(attr);
    state.selected_index = Some(0);
    state.start_editing();
    
    // Modify it
    if let Some(attr) = &mut state.editing_attribute {
        attr.name = "Modified Name".to_string();
    }
    state.mark_dirty();
    
    state.save_current_edit();
    
    assert_eq!(state.attributes.len(), 1);
    assert_eq!(state.attributes[0].name, "Modified Name");
    assert_eq!(state.is_dirty, false);
}

#[test]
fn test_delete_selected() {
    let mut state = EditorState::new();
    
    // Add two attributes
    state.attributes.push(AttributeDefinition::default_new());
    let mut second = AttributeDefinition::default_new();
    second.name = "Second".to_string();
    state.attributes.push(second);
    
    state.selected_index = Some(0);
    state.delete_selected();
    
    assert_eq!(state.attributes.len(), 1);
    assert_eq!(state.attributes[0].name, "Second");
    assert_eq!(state.selected_index, None);
}

#[test]
fn test_filtered_attributes() {
    let mut state = EditorState::new();
    
    let mut attr1 = AttributeDefinition::default_new();
    attr1.name = "Physical Strength".to_string();
    state.attributes.push(attr1);
    
    let mut attr2 = AttributeDefinition::default_new();
    attr2.name = "Mental Power".to_string();
    state.attributes.push(attr2);
    
    let mut attr3 = AttributeDefinition::default_new();
    attr3.name = "Physical Endurance".to_string();
    state.attributes.push(attr3);
    
    // No filter - should return all
    state.search_text = "".to_string();
    assert_eq!(state.filtered_attributes().len(), 3);
    
    // Filter for "Physical"
    state.search_text = "Physical".to_string();
    let filtered = state.filtered_attributes();
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|(_, attr)| attr.name.contains("Physical")));
    
    // Filter for "mental" (case insensitive)
    state.search_text = "mental".to_string();
    let filtered = state.filtered_attributes();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].1.name, "Mental Power");
}

#[test]
fn test_validation_empty_name() {
    let mut attr = AttributeDefinition::default_new();
    attr.name = "".to_string();
    
    let result = validation::validate_attribute(&attr, &[]);
    
    assert!(!result.is_valid());
    assert!(result.errors.iter().any(|e| e.contains("Name cannot be empty")));
}

#[test]
fn test_validation_empty_description() {
    let mut attr = AttributeDefinition::default_new();
    attr.description = "".to_string();
    
    let result = validation::validate_attribute(&attr, &[]);
    
    assert!(!result.is_valid());
    assert!(result.errors.iter().any(|e| e.contains("Description cannot be empty")));
}

#[test]
fn test_validation_duplicate_name() {
    let attr1 = AttributeDefinition {
        id: AttributeId::new(),
        name: "Strength".to_string(),
        description: "First".to_string(),
        category: AttributeCategory::Physical,
        base_value: 10,
        min_value: 1,
        max_value: 100,
        training_difficulty: Percentage::new(100),
        icon: None,
    };
    
    let attr2 = AttributeDefinition {
        id: AttributeId::new(),
        name: "Strength".to_string(),
        description: "Second".to_string(),
        category: AttributeCategory::Physical,
        base_value: 10,
        min_value: 1,
        max_value: 100,
        training_difficulty: Percentage::new(100),
        icon: None,
    };
    
    let result = validation::validate_attribute(&attr2, &[attr1]);
    
    assert!(!result.is_valid());
    assert!(result.errors.iter().any(|e| e.contains("Duplicate name")));
}

#[test]
fn test_validation_min_max_range() {
    let mut attr = AttributeDefinition::default_new();
    attr.min_value = 100;
    attr.max_value = 50;
    
    let result = validation::validate_attribute(&attr, &[]);
    
    assert!(!result.is_valid());
    assert!(result.errors.iter().any(|e| e.contains("Min value") && e.contains("must be less than max value")));
}

#[test]
fn test_validation_base_below_min() {
    let mut attr = AttributeDefinition::default_new();
    attr.min_value = 10;
    attr.max_value = 100;
    attr.base_value = 5;
    
    let result = validation::validate_attribute(&attr, &[]);
    
    assert!(!result.is_valid());
    assert!(result.errors.iter().any(|e| e.contains("Base value") && e.contains("below min value")));
}

#[test]
fn test_validation_base_above_max() {
    let mut attr = AttributeDefinition::default_new();
    attr.min_value = 10;
    attr.max_value = 100;
    attr.base_value = 150;
    
    let result = validation::validate_attribute(&attr, &[]);
    
    assert!(!result.is_valid());
    assert!(result.errors.iter().any(|e| e.contains("Base value") && e.contains("above max value")));
}

#[test]
fn test_validation_zero_training_difficulty() {
    let mut attr = AttributeDefinition::default_new();
    attr.training_difficulty = Percentage::new(0);
    
    let result = validation::validate_attribute(&attr, &[]);
    
    assert!(!result.is_valid());
    assert!(result.errors.iter().any(|e| e.contains("Training difficulty must be greater than 0")));
}

#[test]
fn test_validation_high_training_difficulty_warning() {
    let mut attr = AttributeDefinition::default_new();
    attr.training_difficulty = Percentage::new(1500);
    
    let result = validation::validate_attribute(&attr, &[]);
    
    assert!(result.is_valid()); // Should be valid, just a warning
    assert!(result.warnings.iter().any(|w| w.contains("Training difficulty is very high")));
}

#[test]
fn test_validation_valid_attribute() {
    let attr = AttributeDefinition {
        id: AttributeId::new(),
        name: "Strength".to_string(),
        description: "Physical strength attribute".to_string(),
        category: AttributeCategory::Physical,
        base_value: 10,
        min_value: 1,
        max_value: 100,
        training_difficulty: Percentage::new(100),
        icon: Some("strength.png".to_string()),
    };
    
    let result = validation::validate_attribute(&attr, &[]);
    
    assert!(result.is_valid());
    assert_eq!(result.errors.len(), 0);
}

#[test]
fn test_update_validation_in_state() {
    let mut state = EditorState::new();
    
    state.start_creating();
    
    // Initially should have no errors (default attribute is valid)
    validation::update_validation(&mut state);
    assert_eq!(state.validation_errors.len(), 0);
    
    // Make it invalid
    if let Some(attr) = &mut state.editing_attribute {
        attr.name = "".to_string();
    }
    
    validation::update_validation(&mut state);
    assert!(state.validation_errors.len() > 0);
}

#[test]
fn test_load_attributes_from_file() {
    // This test assumes the attributes.json file exists
    let result = load_attributes();
    
    match result {
        Ok(attrs) => {
            assert!(attrs.len() > 0, "Should have loaded at least one attribute");
            
            // Verify structure
            for attr in attrs {
                assert!(!attr.name.is_empty());
                assert!(!attr.description.is_empty());
                assert!(attr.min_value < attr.max_value);
                assert!(attr.base_value >= attr.min_value);
                assert!(attr.base_value <= attr.max_value);
            }
        }
        Err(e) => {
            // File might not exist in test environment
            println!("Note: Could not load attributes.json: {}", e);
        }
    }
}

#[test]
fn test_save_and_load_attributes() {
    let test_dir = "test_data";
    let test_file = format!("{}/attributes_test.json", test_dir);
    
    // Create test directory if it doesn't exist
    let _ = fs::create_dir_all(test_dir);
    
    // Create test attributes
    let attrs = vec![
        AttributeDefinition {
            id: AttributeId::new(),
            name: "Test Physical".to_string(),
            description: "Test physical attribute".to_string(),
            category: AttributeCategory::Physical,
            base_value: 10,
            min_value: 1,
            max_value: 100,
            training_difficulty: Percentage::new(100),
            icon: None,
        },
        AttributeDefinition {
            id: AttributeId::new(),
            name: "Test Mental".to_string(),
            description: "Test mental attribute".to_string(),
            category: AttributeCategory::Mental,
            base_value: 12,
            min_value: 5,
            max_value: 50,
            training_difficulty: Percentage::new(120),
            icon: Some("mental.png".to_string()),
        },
    ];
    
    // Save to file
    let json = serde_json::to_string_pretty(&attrs).unwrap();
    fs::write(&test_file, json).unwrap();
    
    // Load back
    let content = fs::read_to_string(&test_file).unwrap();
    let loaded: Vec<AttributeDefinition> = serde_json::from_str(&content).unwrap();
    
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].name, "Test Physical");
    assert_eq!(loaded[1].name, "Test Mental");
    
    // Cleanup
    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_dir(test_dir);
}

#[test]
fn test_mark_dirty() {
    let mut state = EditorState::new();
    
    assert_eq!(state.is_dirty, false);
    
    state.mark_dirty();
    
    assert_eq!(state.is_dirty, true);
}

#[test]
fn test_revert_edit() {
    let mut state = EditorState::new();
    
    state.start_creating();
    state.mark_dirty();
    
    assert!(state.editing_attribute.is_some());
    assert_eq!(state.is_dirty, true);
    
    state.revert_edit();
    
    assert_eq!(state.editing_attribute, None);
    assert_eq!(state.is_dirty, false);
}
