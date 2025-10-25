# Phase 7: Bevy UI Prototype

## Overview
Build a native desktop UI using the Bevy game engine to interact with the Timeloop game server, styled with the Visual Studio Code Cobalt theme.

## Technology Stack

### Frontend
- **Bevy 0.14** - Game engine and UI framework
- **bevy_egui 0.28** - Immediate mode GUI for rapid prototyping
- **reqwest 0.12** - HTTP client for API calls
- **tokio 1.0** - Async runtime integration

### Architecture
- Bevy ECS for UI state management
- Async communication with Rust backend API
- Event-driven updates between UI and server

## Visual Design - Cobalt Theme

### Color Palette (from VS Code Cobalt)
```rust
// Background colors
const BG_DARK: Color = Color::rgb(0.09, 0.11, 0.15);        // #193549 (main background)
const BG_MEDIUM: Color = Color::rgb(0.12, 0.15, 0.20);      // #1F2D3A (panels)
const BG_LIGHT: Color = Color::rgb(0.15, 0.19, 0.25);       // #27394F (hover/select)

// Text colors
const TEXT_PRIMARY: Color = Color::rgb(1.0, 1.0, 1.0);      // #FFFFFF (main text)
const TEXT_SECONDARY: Color = Color::rgb(0.8, 0.8, 0.8);    // #CCCCCC (secondary)
const TEXT_MUTED: Color = Color::rgb(0.5, 0.6, 0.7);        // #8A99A6 (muted)

// Accent colors
const ACCENT_BLUE: Color = Color::rgb(0.31, 0.59, 0.84);    // #5095D6 (primary accent)
const ACCENT_CYAN: Color = Color::rgb(0.0, 0.8, 0.8);       // #00CCCC (highlights)
const ACCENT_ORANGE: Color = Color::rgb(1.0, 0.6, 0.0);     // #FF9D00 (warnings)
const ACCENT_GREEN: Color = Color::rgb(0.6, 0.8, 0.2);      // #99CC33 (success)
const ACCENT_PURPLE: Color = Color::rgb(0.8, 0.4, 0.8);     // #CC66CC (special)

// Progress/Status colors
const PROGRESS_BG: Color = Color::rgb(0.2, 0.25, 0.3);      // Progress bar background
const PROGRESS_FILL: Color = ACCENT_BLUE;                   // Progress bar fill
const TRAINING_INDICATOR: Color = ACCENT_ORANGE;            // Training active
```

### Typography
- **Font**: Consolas (monospace font from screenshot)
- **Primary text size**: 16px
- **Secondary text size**: 14px
- **Headers**: 20px bold
- **Monospace throughout** for consistent game feel

### UI Style
- Dark theme matching VS Code Cobalt
- Subtle borders and separators
- Smooth gradients on buttons
- Glow effects on interactive elements
- Rounded corners (4px radius)
- Drop shadows for depth

## UI Features

### 1. Main Screens/States
Using Bevy's state system:
- `MainMenu` - Welcome screen
- `CharacterCreation` - Create new Timelooper
- `GameDashboard` - Main game view

### 2. Game Dashboard Layout

**Top Bar (40px height):**
- Character name (white text)
- Current Loop | Current Tick (cyan accent)
- Save Game button (blue accent)

**Left Panel (220px):**
- **Attributes Section:**
  - Section header with blue underline
  - List of attributes with progress bars
  - Current values displayed (white text)
  - Training indicators (🎯 orange when active)
  - Progress bars with blue fill on dark background

**Center Panel:**
- **Character Portrait Area** (placeholder for future)
- **Active Training Display:**
  - Card with dark background
  - "Training: Physical (450 ticks)" in orange
  
- **Action Buttons:**
  - Blue gradient buttons with hover effect
  - "Advance 10 Ticks"
  - "Advance 100 Ticks"
  - "Start Training →" (dropdown)
  - "Stop Training"

**Right Panel (220px):**
- **Affinities List:**
  - Section header
  - Icon + Name + Mastery Level
  - Cards with dark background
  - Purple accent for special affinities
  - "Acquire New Affinity" button (green accent)
  
- **Active Effects:**
  - Effect name + duration
  - Modifier display with colored indicators
  - Cyan text for positive effects
  - Orange text for negative effects

**Bottom Bar (30px):**
- Status messages / notifications (muted text)
- API connection status (green dot when connected)

## Project Structure
```
src/
├── main.rs              # Bevy client entry point (renamed)
├── lib.rs               # Existing lib (keep for shared code)
├── bin/
│   └── server.rs        # Server binary (moved from main.rs)
├── client/
│   ├── mod.rs           # Client module root
│   ├── api.rs           # API client wrapper
│   ├── state.rs         # Bevy resources/components
│   ├── theme.rs         # Cobalt theme colors and styles
│   ├── systems/
│   │   ├── mod.rs
│   │   ├── ui.rs        # UI rendering systems
│   │   ├── input.rs     # Input handling
│   │   └── network.rs   # API communication
│   └── components.rs    # UI components
└── assets/
    └── fonts/
        └── consolas.ttf # Consolas font
```

## Bevy Resources & Components

### Resources
```rust
// Game state from server
pub struct GameStateResource {
    pub character_name: String,
    pub current_loop: u32,
    pub current_tick: u64,
    pub attributes: Vec<AttributeDisplay>,
    pub affinities: Vec<AffinityDisplay>,
    pub training_attribute: Option<AttributeId>,
}

// API client
pub struct ApiClient {
    base_url: String,
    client: reqwest::Client,
}

// Definitions cache
pub struct GameDefinitions {
    pub attributes: Vec<AttributeDefinition>,
    pub affinities: Vec<AffinityDefinition>,
    pub effects: Vec<EffectDefinition>,
}

// UI state
pub struct UiState {
    pub selected_attribute: Option<AttributeId>,
    pub selected_affinity: Option<AffinityId>,
    pub status_message: String,
}

// Theme resource
pub struct CobaltTheme {
    pub bg_dark: Color,
    pub bg_medium: Color,
    pub text_primary: Color,
    // ... all theme colors
}
```

### Components
```rust
// UI button markers
pub struct AdvanceTickButton(pub u64);
pub struct StartTrainingButton;
pub struct StopTrainingButton;
pub struct AcquireAffinityButton;
pub struct SaveGameButton;

// Text display markers
pub struct CharacterNameText;
pub struct TickDisplayText;
pub struct AttributeValueText(pub AttributeId);
```

## Implementation Steps

### Step 1: Project Setup (30 min)
- Add Bevy dependencies to `Cargo.toml`
- Create separate binary targets:
  - `timeloop-server` - Existing REST API server
  - `timeloop-client` - New Bevy UI application
- Move `main.rs` to `src/bin/server.rs`
- Create new `src/main.rs` for Bevy client
- Set up basic Bevy app with window
- Configure window (1280x720, Cobalt dark background)

### Step 2: Theme Module (15 min)
- Create `client/theme.rs` with Cobalt color palette
- Define color constants from VS Code Cobalt theme
- Create helper functions for styled UI elements
- Load Consolas font (or fallback monospace)

### Step 3: API Client Module (45 min)
- Create `client/api.rs` with async API wrapper
- Implement functions for all endpoints
- Handle errors and connection issues
- Channel-based communication between Bevy and async runtime
- Test API connectivity

### Step 4: Game State Management (30 min)
- Define Bevy resources for game state
- Create systems to poll/update state from API
- Event system for API responses
- State synchronization logic

### Step 5: Main Menu UI (30 min)
- Simple menu with Cobalt styling
- "New Game" button with blue accent
- Text input for character name (dark input field)
- "Create Character" button
- Transition to game dashboard

### Step 6: Game Dashboard Layout (1 hour)
- Top bar with character info (Cobalt colors)
- Left panel with attributes list (styled cards)
- Center panel with actions (gradient buttons)
- Right panel with affinities (purple accents)
- Bottom status bar (muted text)
- Use bevy_egui with custom Cobalt styling

### Step 7: Interactive Elements (1 hour)
- Button click handlers with hover effects
- Dropdown/selection for training
- Modal for affinity selection (overlay with dark bg)
- Progress bars with blue gradient fill
- Visual feedback for actions (glow effects)
- Loading indicators with cyan accent

### Step 8: Network Systems (45 min)
- System to send API requests
- System to handle responses
- Auto-refresh system (poll every 1-2 sec)
- Error handling and retry logic
- Connection status indicator

### Step 9: Polish & Visual Effects (45 min)
- Smooth transitions with easing
- Hover effects on buttons (blue glow)
- Success notifications (green)
- Error notifications (orange)
- Keyboard shortcuts
- Animations for state changes

## Cargo.toml Changes

```toml
[package]
name = "timeloop"
version = "0.1.0"
edition = "2021"

[lib]
name = "timeloop"
path = "src/lib.rs"

[[bin]]
name = "timeloop-server"
path = "src/bin/server.rs"

[[bin]]
name = "timeloop-client"
path = "src/main.rs"

[dependencies]
# Existing server dependencies
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
tokio = { version = "1.0", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "fs", "trace"] }
tracing = "0.1"
tracing-subscriber = "0.3"
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"

# Bevy client dependencies
bevy = { version = "0.14", features = ["default"] }
bevy_egui = "0.28"
reqwest = { version = "0.12", features = ["json"] }

[dev-dependencies]
tower = { version = "0.4", features = ["util"] }
http-body-util = "0.1"
```

## Key Bevy Systems

### Startup Systems
- `setup_theme` - Load Cobalt theme colors
- `setup_ui` - Create initial UI hierarchy
- `setup_api_client` - Initialize HTTP client
- `load_definitions` - Fetch game definitions
- `load_fonts` - Load Consolas font

### Update Systems
- `poll_game_state` - Periodic API polling
- `handle_button_clicks` - Process UI interactions
- `update_attribute_displays` - Refresh attribute UI
- `update_affinity_displays` - Refresh affinity UI
- `handle_api_responses` - Process async responses
- `animate_ui_elements` - Smooth transitions

### State-Specific Systems
- `main_menu_ui` - Render main menu (MainMenu state)
- `game_dashboard_ui` - Render dashboard (GameDashboard state)

## Visual Design (Bevy UI with Cobalt Theme)

```
┌─────────────────────────────────────────────────────────────┐
│  TIMELOOP              Loop: 1  |  Tick: 4500      [Save]   │ ← #193549 bg
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  Attributes          │   Active Training                     │
│  ────────────       │   ╔══════════════════════════╗        │
│                      │   ║ Physical (450 ticks)    ║        │ ← #1F2D3A card
│  ╔═══════════════╗  │   ╚══════════════════════════╝        │
│  ║ Physical: 15  ║  │                                        │
│  ║ ████████░░    ║  │   ┌────────────────────┐             │
│  ║ (Training 🎯) ║  │   │ Advance 10 Ticks   │ ← #5095D6    │
│  ╚═══════════════╝  │   └────────────────────┘             │
│                      │   ┌────────────────────┐             │
│  ╔═══════════════╗  │   │ Advance 100 Ticks  │             │
│  ║ Mental: 12    ║  │   └────────────────────┘             │
│  ║ ██████░░░     ║  │                                        │
│  ╚═══════════════╝  │   ┌────────────────────┐             │
│                      │   │ Start Training ▼   │             │
│  ╔═══════════════╗  │   └────────────────────┘             │
│  ║ Social: 10    ║  │                                        │
│  ║ █████░░░░     ║  │   Affinities                          │
│  ╚═══════════════╝  │   ────────────                        │
│                      │   ⚔️ Combat (Lv 1)                    │
│                      │   💼 Business (Lv 2)  ← #CC66CC       │
│                      │                                        │
│                      │   ┌────────────────────┐             │
│                      │   │ Acquire New +      │ ← #99CC33    │
│                      │   └────────────────────┘             │
├─────────────────────────────────────────────────────────────┤
│  ● Status: Training Physical... (+1 progress)               │ ← #8A99A6
└─────────────────────────────────────────────────────────────┘
```

## Communication Architecture

```
Bevy App (Client)
    ↓
Event: AdvanceTickRequested(10)
    ↓
System: send_api_request
    ↓
Async Channel → tokio::spawn
    ↓
reqwest POST http://127.0.0.1:3000/api/game/tick
    ↓
Response ← Server API
    ↓
Event: ApiResponse(TickAdvanced)
    ↓
System: update_game_state_resource
    ↓
UI Rendering Systems update display (Cobalt styled)
```

## Estimated Implementation Time
- Step 1: Project setup - 30 min
- Step 2: Theme module - 15 min
- Step 3: API client - 45 min
- Step 4: State management - 30 min
- Step 5: Main menu - 30 min
- Step 6: Dashboard layout - 60 min
- Step 7: Interactive elements - 60 min
- Step 8: Network systems - 45 min
- Step 9: Polish & effects - 45 min

**Total: ~6 hours for working prototype with Cobalt theme**

## Success Criteria
- ✅ Bevy window opens with Cobalt dark theme
- ✅ Consolas font loaded and used throughout
- ✅ Main menu with styled input and buttons
- ✅ Can create new character via form
- ✅ Dashboard displays with Cobalt color palette
- ✅ All UI elements match VS Code Cobalt theme
- ✅ Can click buttons to advance time
- ✅ Can start/stop training via UI
- ✅ Progress bars with blue gradient
- ✅ Hover effects with blue glow
- ✅ UI updates reflect server state changes
- ✅ Smooth 60 FPS rendering
- ✅ Responsive layout

## Future Enhancements
- Custom shaders for glow effects
- Particle effects on actions
- Sound effects and music
- More detailed character visualization
- Animation system for transitions
- Settings menu for theme customization
