# Game Object Editor Plan

## Overview
Build an editor tool for creating, modifying, and managing game object definitions (Attributes, Affinities, Effects) for the Timeloop game. The editor will provide a user-friendly interface for game designers to manage game data without directly editing JSON files.

## Goals
- Enable non-programmers to create and modify game objects
- Validate game data to prevent invalid configurations
- Support version control friendly output (formatted JSON)
- Provide live preview of game objects
- Export/Import game definitions
- Maintain consistency across all game objects

## Technology Stack

### Option A: Bevy-based Editor (Recommended)
**Advantages:**
- Reuse existing Bevy client infrastructure
- Consistent UI theme (Cobalt)
- Native desktop application
- Can share code with main game client
- Integrated with existing data models

**Disadvantages:**
- More initial setup
- Requires Bevy knowledge

### Option B: Web-based Editor
**Advantages:**
- Platform independent
- Easy deployment
- Familiar web technologies
- Can be hosted remotely

**Disadvantages:**
- Separate tech stack
- Need additional backend
- Different UI paradigm

**Decision: Start with Option A (Bevy-based) for consistency**

## Game Object Types

### 1. Attribute Definitions
**Fields to edit:**
- ID (UUID, auto-generated or manual)
- Name (string)
- Description (text)
- Category (enum: Physical, Mental, Social)
- Base Value (u32)
- Min Value (u32)
- Max Value (u32)
- Training Difficulty (percentage)
- Icon (file path/name)

**Validation rules:**
- Name must be unique
- Base value must be between min and max
- Min < Max
- Training difficulty > 0

### 2. Affinity Definitions
**Fields to edit:**
- ID (UUID)
- Name (string)
- Description (text)
- Lore Text (multiline text)
- Attribute Bonuses (map: AttributeId → u32)
- Mastery Benefits (map: level → description)
- Icon (file path/name)

**Validation rules:**
- Name must be unique
- Attribute bonuses must reference valid attributes
- Mastery levels must be positive integers
- Cannot have duplicate mastery levels

### 3. Effect Definitions
**Fields to edit:**
- ID (UUID)
- Name (string)
- Description (text)
- Property (enum: what it affects)
- Modification (how it modifies)
  - Type (Add, Multiply, Set, etc.)
  - Value (number)
- Default Duration (optional u64)
- Default Conditions (list of strings)
- Stacking Behavior (enum)
- Priority (i32)
- Icon (file path/name)
- Visual Effect (string)

**Validation rules:**
- Name must be unique
- Property must be valid
- Modification value must be appropriate for type
- Priority can be any integer (negative for pre-modifiers)

## Editor Features

### Core Features (Phase 1)
1. **List View**
   - Display all objects of selected type
   - Search/filter by name or properties
   - Sort by various fields
   - Quick preview on hover

2. **Create New Object**
   - Template selection (copy from existing)
   - Step-by-step wizard for complex objects
   - Auto-generate UUIDs
   - Validation on each field

3. **Edit Existing Object**
   - Form-based editing
   - Real-time validation
   - Preview changes
   - Undo/Redo support

4. **Delete Object**
   - Confirmation dialog
   - Check for dependencies (warn if used by other objects)
   - Soft delete with recovery option

5. **Save/Export**
   - Save individual objects
   - Save all changes
   - Export to formatted JSON
   - Backup previous versions

### Advanced Features (Phase 2)
1. **Dependency Tracking**
   - Show which objects reference this object
   - Visualize relationships
   - Prevent deletion of referenced objects
   - Cascade updates option

2. **Validation System**
   - Custom validation rules
   - Warning vs Error distinction
   - Batch validation for all objects
   - Export validation report

3. **Templates & Presets**
   - Save custom templates
   - Import community templates
   - Preset configurations (e.g., "Warrior build", "Mage build")

4. **Bulk Operations**
   - Bulk edit similar objects
   - Mass import from CSV/spreadsheet
   - Batch rename/update
   - Clone multiple objects

5. **Version Control Integration**
   - Git-friendly JSON formatting
   - Diff viewer for changes
   - Commit message generation
   - Branch/merge support awareness

### Future Features (Phase 3)
1. **Visual Effect Preview**
   - Show how attributes/effects modify character
   - Simulate progression over time
   - Combat calculator
   - Build optimizer

2. **Balance Tools**
   - Power level calculator
   - Cost/benefit analysis
   - Progression curves
   - Statistical analysis

3. **Localization Support**
   - Multi-language strings
   - Translation management
   - Export for translators

## Project Structure

```
src/
├── bin/
│   ├── server.rs          # Game server
│   ├── client.rs          # Game client (renamed from main.rs)
│   └── editor.rs          # NEW: Game object editor
├── editor/
│   ├── mod.rs             # Editor module root
│   ├── state.rs           # Editor state management
│   ├── validation.rs      # Validation rules
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── layout.rs      # Main editor layout
│   │   ├── list_view.rs   # Object list view
│   │   ├── detail_view.rs # Object detail/edit view
│   │   ├── forms.rs       # Form components
│   │   └── dialogs.rs     # Confirmation dialogs, wizards
│   ├── io/
│   │   ├── mod.rs
│   │   ├── loader.rs      # Load JSON definitions
│   │   ├── saver.rs       # Save JSON definitions
│   │   └── validator.rs   # Validate JSON structure
│   └── operations/
│       ├── mod.rs
│       ├── create.rs      # Create operations
│       ├── edit.rs        # Edit operations
│       ├── delete.rs      # Delete operations
│       └── dependencies.rs # Dependency tracking
```

## UI Layout (Bevy/egui with Cobalt Theme)

```
┌──────────────────────────────────────────────────────────────────┐
│  Timeloop Game Object Editor           [File] [Edit] [View] [?]  │
├──────────────────────────────────────────────────────────────────┤
│                                                                    │
│  ┌─────────────┐  ┌──────────────────────────────────────────┐  │
│  │ Object Type │  │  List View                               │  │
│  ├─────────────┤  ├──────────────────────────────────────────┤  │
│  │             │  │  Search: [_________________] [Filter ▼] │  │
│  │ Attributes  │  │                                          │  │
│  │ Affinities  │  │  ╔═══════════════════════════════════╗  │  │
│  │ Effects     │  │  ║ Physical                          ║  │  │
│  │             │  │  ║ Base: 10 | Training: 100         ║  │  │
│  │             │  │  ╚═══════════════════════════════════╝  │  │
│  │             │  │                                          │  │
│  │             │  │  ╔═══════════════════════════════════╗  │  │
│  │             │  │  ║ Mental                            ║  │  │
│  │             │  │  ║ Base: 10 | Training: 100         ║  │  │
│  │             │  │  ╚═══════════════════════════════════╝  │  │
│  │             │  │                                          │  │
│  │             │  │  [+ New Attribute]                      │  │
│  └─────────────┘  └──────────────────────────────────────────┘  │
│                                                                    │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Detail View - Editing: Physical                          │  │
│  ├───────────────────────────────────────────────────────────┤  │
│  │                                                            │  │
│  │  Name:        [Physical                              ]    │  │
│  │  Description: [Physical prowess, strength, endurance ]    │  │
│  │  Category:    [Physical ▼]                               │  │
│  │  Base Value:  [10    ]  (Min: 1   Max: 100   )          │  │
│  │  Training:    [100   ]  (Difficulty percentage)          │  │
│  │  Icon:        [physical.png            ] [Browse...]     │  │
│  │                                                            │  │
│  │  ✓ Valid  │  [Save Changes]  [Revert]  [Delete]         │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                    │
├──────────────────────────────────────────────────────────────────┤
│  Status: 3 attributes, 3 affinities, 3 effects loaded | Unsaved │
└──────────────────────────────────────────────────────────────────┘
```

## Implementation Strategy

**Iterative Approach**: Build complete functionality for one object type at a time, starting with the simplest (Attributes), then expand to others. This ensures we have a working editor quickly and can learn from each iteration.

### Phase 1A: Attributes Editor (Complete Feature Set)

This phase delivers a fully functional editor for Attributes only. Once this works perfectly, we'll clone the pattern for other object types.

#### Step 1: Project Setup (30 min)
- Create `src/bin/editor.rs` binary target
- Add to `Cargo.toml` as new binary
- Set up basic Bevy app with window
- Apply Cobalt theme
- Create editor module structure

#### Step 2: Attribute Data Loading (30 min)
- Create `AttributeLoader` specific to attributes
- Load `game_data/core/attributes.json`
- Parse into `Vec<AttributeDefinition>`
- Display count in status bar
- Error handling for corrupted JSON
- Backup system before any modifications

#### Step 3: Attribute List View UI (1 hour)
- Single panel showing list of attributes
- Display key properties per attribute:
  - Name
  - Category (Physical/Mental/Social)
  - Base value
  - Training difficulty
- Click to select attribute
- Search/filter by name
- Sort by name, category, or base value
- Visual indicator for selected item

#### Step 4: Attribute Detail View UI (1.5 hours)
- Form layout for selected attribute
- **Fields:**
  - ID (display only, UUID)
  - Name (text input, required)
  - Description (multiline text, required)
  - Category (dropdown: Physical, Mental, Social)
  - Base Value (number input, default 10)
  - Min Value (number input, default 1)
  - Max Value (number input, default 100)
  - Training Difficulty (number input, default 100)
  - Icon (text input for filename)
- Validation indicators next to each field
- "Valid" or error message display at bottom

#### Step 5: Create Attribute Operation (1 hour)
- "New Attribute" button at top of list
- Generate new UUID automatically
- Pre-fill with sensible defaults:
  - Name: "New Attribute"
  - Description: ""
  - Category: Physical
  - Base: 10, Min: 1, Max: 100
  - Training: 100
  - Icon: ""
- Open in detail view immediately
- Validation on required fields
- Add to list on save (doesn't persist until Save All)

#### Step 6: Edit Attribute Operation (45 min)
- Load selected attribute into form
- Track changes (dirty flag indicator)
- Real-time validation as user types
- Save changes to memory (not file yet)
- Revert button to undo changes
- Visual indicator showing unsaved changes

#### Step 7: Delete Attribute Operation (30 min)
- Delete button with confirmation dialog
- "Are you sure you want to delete [Name]?"
- Remove from memory (not file yet)
- Clear detail view after deletion
- Can be undone before "Save All"

#### Step 8: Attribute Save System (1 hour)
- "Save All" button in top bar
- Write all attributes to `attributes.json`
- Pretty-print JSON with consistent formatting:
  - 2-space indentation
  - Sorted keys
  - UTF-8 encoding
- Atomic writes (write to temp, then rename)
- Backup previous version to `attributes.json.backup`
- Validation before save (prevent saving invalid data)
- Success/failure message

#### Step 9: Attribute Validation System (1 hour)
- Real-time field validation:
  - Name not empty
  - Description not empty
  - Min < Max
  - Base between Min and Max
  - Training difficulty > 0
- Cross-field validation
- Uniqueness validation (no duplicate names)
- Warning/Error indicators on fields
- Prevent save if errors present
- Display all issues in validation panel
- Color-coded messages (red errors, orange warnings)

#### Step 10: Polish & Testing (45 min)
- Keyboard shortcuts:
  - Ctrl+S: Save All
  - Ctrl+N: New Attribute
  - Delete: Delete selected
  - Escape: Clear selection
- Better visual feedback on actions
- Test all edge cases
- Ensure no data loss scenarios
- Test with actual game data
- Documentation comments

**Phase 1A Total: ~8.5 hours**

### Phase 1B: Affinities Editor (Clone & Adapt)

Once attributes editor is working perfectly, implement the same features for Affinities:

#### Step 11: Affinities Support (2 hours)
- Add Affinity tab/section in UI
- Load `affinities.json`
- List view for affinities
- Detail view with affinity-specific fields:
  - Attribute Bonuses (map editor)
  - Mastery Benefits (list editor)
  - Lore Text (larger text area)
- Create/Edit/Delete operations
- Validation specific to affinities
- Save system for affinities

**Key Differences from Attributes:**
- More complex data structures (maps, lists)
- References to AttributeIds (validation needed)
- Longer text fields (lore)

### Phase 1C: Effects Editor (Clone & Adapt)

Finally, add Effects support:

#### Step 12: Effects Support (2.5 hours)
- Add Effects tab/section in UI
- Load `effects.json`
- List view for effects
- Detail view with effect-specific fields:
  - Property (enum dropdown)
  - Modification (type + value)
  - Default Duration (optional)
  - Conditions (list of strings)
  - Stacking Behavior (enum)
  - Priority (integer)
- Create/Edit/Delete operations
- Validation specific to effects
- Save system for effects

**Key Differences:**
- Most complex data structures
- Nested enums and values
- Optional fields
- More validation rules

### Phase 2: Advanced Features (Future)

#### Step 10: Dependency Tracking (2 hours)
- Build dependency graph
- Show "Used by" section
- Prevent deletion of referenced objects
- Update cascade options
- Visualize relationships

#### Step 11: Templates & Bulk Operations (2 hours)
- Save object as template
- Clone object feature
- Bulk edit selected objects
- Import from CSV
- Export to CSV

#### Step 12: Polish & UX (2 hours)
- Keyboard shortcuts
- Context menus (right-click)
- Drag-and-drop reordering
- Color coding by category
- Icons for object types
- Better visual feedback

## Cargo.toml Changes

```toml
[[bin]]
name = "timeloop-editor"
path = "src/bin/editor.rs"

# Dependencies remain same as client
# (Bevy, bevy_egui, serde, serde_json, uuid)
```

## Validation Rules Implementation

```rust
pub trait Validator {
    fn validate(&self, context: &ValidationContext) -> ValidationResult;
}

pub struct ValidationContext {
    all_attributes: HashMap<AttributeId, AttributeDefinition>,
    all_affinities: HashMap<AffinityId, AffinityDefinition>,
    all_effects: HashMap<EffectDefinitionId, EffectDefinition>,
}

pub struct ValidationResult {
    errors: Vec<String>,
    warnings: Vec<String>,
    is_valid: bool,
}

// Example: Attribute validation
impl Validator for AttributeDefinition {
    fn validate(&self, context: &ValidationContext) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Name validation
        if self.name.is_empty() {
            result.add_error("Name cannot be empty");
        }
        
        // Value range validation
        if self.min_value >= self.max_value {
            result.add_error("Min value must be less than max value");
        }
        
        if self.base_value < self.min_value || self.base_value > self.max_value {
            result.add_error("Base value must be between min and max");
        }
        
        // Training difficulty validation
        if self.training_difficulty == 0 {
            result.add_error("Training difficulty must be greater than 0");
        }
        
        if self.training_difficulty > 1000 {
            result.add_warning("Training difficulty is very high (>1000%)");
        }
        
        // Uniqueness check
        for attr in context.all_attributes.values() {
            if attr.id != self.id && attr.name == self.name {
                result.add_error(format!("Duplicate name: {}", self.name));
            }
        }
        
        result
    }
}
```

## File Format Standards

### JSON Formatting
- 2-space indentation
- Sorted keys for consistent diffs
- One object per line in arrays where reasonable
- UTF-8 encoding
- Unix line endings (LF)

### Example Output
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "name": "Physical",
    "description": "Physical prowess, strength, and endurance",
    "category": "Physical",
    "base_value": 10,
    "min_value": 1,
    "max_value": 100,
    "training_difficulty": 100,
    "icon": "physical.png"
  }
]
```

## Success Criteria

### Phase 1A: Attributes Editor (Must Have)
- ✅ Can load attributes from JSON
- ✅ Can view list of all attributes
- ✅ Can select and view attribute details
- ✅ Can create new attributes with validation
- ✅ Can edit existing attributes
- ✅ Can delete attributes with confirmation
- ✅ Can save all changes to attributes.json
- ✅ Validation prevents invalid attribute data
- ✅ Backup system creates .backup files
- ✅ Cobalt theme applied consistently
- ✅ Keyboard shortcuts work (Ctrl+S, Ctrl+N, etc.)
- ✅ No data loss possible

### Phase 1B: Affinities Editor (Second Priority)
- ✅ Can load, view, edit affinities
- ✅ Map editor for attribute bonuses
- ✅ List editor for mastery benefits
- ✅ References to attributes validated
- ✅ Save to affinities.json

### Phase 1C: Effects Editor (Third Priority)
- ✅ Can load, view, edit effects
- ✅ Complex nested structure editing
- ✅ Enum handling for properties
- ✅ Optional field support
- ✅ Save to effects.json

### Phase 2: Multi-Object Features (Nice to Have)
- ✅ Dependency tracking shows relationships
- ✅ Cannot delete referenced objects
- ✅ Cross-object validation
- ✅ Template system works
- ✅ Bulk operations functional

### Phase 3: Advanced Polish (Future)
- ✅ Import/Export CSV
- ✅ Preview system shows effects
- ✅ Balance tools provide feedback
- ✅ Undo/Redo history
- ✅ Context menus
- ✅ Drag-and-drop

## Estimated Implementation Time

### Phase 1A: Attributes Editor (Complete)
- Step 1: Project setup - 30 min
- Step 2: Attribute loading - 30 min
- Step 3: List view - 60 min
- Step 4: Detail view - 90 min
- Step 5: Create operation - 60 min
- Step 6: Edit operation - 45 min
- Step 7: Delete operation - 30 min
- Step 8: Save system - 60 min
- Step 9: Validation - 60 min
- Step 10: Polish & testing - 45 min

**Phase 1A Total: ~8.5 hours for complete Attributes editor**

### Phase 1B: Affinities Editor
- Step 11: Affinities support - 120 min

**Phase 1B Total: ~2 hours (leveraging existing code)**

### Phase 1C: Effects Editor
- Step 12: Effects support - 150 min

**Phase 1C Total: ~2.5 hours (leveraging existing code)**

**Overall Phase 1 Total: ~13 hours for all three object types**

### Phase 2: Advanced Features
- Dependency tracking - 120 min
- Templates/Bulk operations - 120 min

**Phase 2 Total: ~4 hours**

**Grand Total: ~17 hours for full-featured editor**

## Risk Mitigation

### Data Loss Prevention
- Auto-save to temp directory
- Backup before any save operation
- Confirmation dialogs for destructive actions
- Undo/Redo system
- Version history

### Invalid Data Prevention
- Comprehensive validation rules
- Prevent save if errors exist
- Dry-run mode for bulk operations
- Schema validation against game models

### User Errors
- Clear error messages
- Inline help text
- Tooltips explaining fields
- Default values for all fields
- Templates for common patterns

## Future Expansion

### Additional Object Types
Once base editor is working, easily extend to:
- Skills
- Masteries  
- Items
- Equipment
- Locations
- NPCs
- Events
- Quests
- Dialogue trees

### Integration Options
- Live reload in running game
- Hot-reload definitions without restart
- Network editor (edit while game server runs)
- Collaborative editing (multiple designers)
- Cloud sync for definitions

### Quality of Life
- Recent files list
- Favorites/bookmarks
- Custom views/filters
- Workspaces (edit multiple files)
- Diff viewer
- Changelog generation
- Documentation export

## Development Approach

### Iterative Development (Recommended)
1. **Week 1**: Complete Phase 1A (Attributes Editor)
   - Get a fully working editor for one object type
   - Test thoroughly with real data
   - Learn what works and what doesn't
   - Gather feedback

2. **Week 2**: Add Phase 1B (Affinities)
   - Apply lessons learned from attributes
   - Refactor common code into reusable components
   - Handle more complex data structures

3. **Week 3**: Add Phase 1C (Effects)
   - Complete the three core object types
   - Ensure all work together
   - Final refactoring

4. **Week 4+**: Phase 2 (Advanced Features)
   - Add dependency tracking
   - Add templates and bulk operations
   - Polish and optimize

### Why Start with Attributes?

**Simplest data structure:**
- Flat fields (no nested maps or lists)
- Few validation rules
- Easy to understand
- Quick to implement

**Immediate value:**
- Currently only 2-3 attributes in game
- Will need many more attributes as game expands
- Manually editing JSON is error-prone
- Foundation for other editors

**Learning opportunity:**
- Establish patterns for other object types
- Test validation framework
- Test save/backup system
- Test UI layout

## Next Steps

### Immediate (Phase 1A - Attributes Editor)
1. Create `src/bin/editor.rs` with basic Bevy app
2. Add Cobalt theme from existing client
3. Implement attribute list view
4. Implement attribute detail/edit view
5. Add create/edit/delete/save operations
6. Add validation system
7. Test thoroughly with `attributes.json`

### Short-term (Phase 1B - Affinities)
1. Refactor common code into reusable components
2. Add affinity-specific UI components
3. Implement map editor for attribute bonuses
4. Implement list editor for mastery benefits
5. Add cross-reference validation

### Medium-term (Phase 1C - Effects)
1. Handle complex nested structures
2. Implement enum editors
3. Add optional field support
4. Complete validation for all types

### Long-term (Phase 2+)
1. Add dependency tracking across object types
2. Implement templates and bulk operations
3. Add import/export capabilities
4. Optimize for large datasets

## Getting Started

To begin Phase 1A immediately:

```bash
# 1. Create editor binary file
# 2. Update Cargo.toml to add timeloop-editor binary
# 3. Copy client theme and basic UI setup
# 4. Focus solely on Attributes
# 5. Don't worry about Affinities or Effects yet

cargo run --bin timeloop-editor
```

This editor will significantly improve the game design workflow and make it easier to iterate on game balance and content. Starting with Attributes ensures we get a working tool quickly while establishing patterns for future expansion.
