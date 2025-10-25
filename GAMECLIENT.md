# Game Client Architecture Plan

## Overview

This document outlines the technical approach for building the timeloop web client.

## Technology Stack

**Technology**
- Bevy game engine

**Communication:**
- REST API for game state operations
- JSON for data serialization
- Fetch API for HTTP requests
- WebSockets (future consideration for real-time features)

**Backend Integration:**
- Existing Rust/Axum backend with JSON API endpoints
- UUID-based entity identification
- File-based save system through API

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Bevy Client                              │
├─────────────────┬─────────────────┬─────────────────────────┤
│   UI Layer      │  Game Logic     │    API Layer            │
│                 │                 │                         │
│ • Bevy components│ • State Mgmt    │ • REST Client           │
│ • 3D engine     │ • Game Flow     │ • JSON Serialization   │
│ • Reusable      │ • Validation    │ • Error Handling        │
│ • Event Handling│ • UI Updates    │ • Save/Load Operations  │
└─────────────────┴─────────────────┴─────────────────────────┘
                            │
                    ┌───────▼───────┐
                    │   HTTP/JSON   │
                    └───────┬───────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                 Rust Backend (Axum)                        │
├─────────────────┬─────────────────┬─────────────────────────┤
│   API Gateway   │  Game Engine    │   Storage Layer         │
│                 │                 │                         │
│ • REST Endpoints│ • Game Logic    │ • File Repository       │
│ • Validation    │ • Simulation    │ • JSON Persistence      │
│ • Serialization │ • Event System  │ • UUID Indexing         │
│ • CORS Handling │ • Rule Engine   │ • Save/Load Operations  │
└─────────────────┴─────────────────┴─────────────────────────┘
```

## Phase Implementation Plan

### Phase 1: Foundation & Timerlooper Creation

#### 1.1 Main Menu System
```
Main Menu
├── New Game
│   ├── Create Timelooper
│   └── Start New Timeloop world
├── Load Game
│   ├── Select Save File
│   └── Resume Game
└── Settings
    ├── Game duration
    └── Interface Options
```
