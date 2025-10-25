Folder structure:

This is the backend implementation of the game and the API gateways. The UI will be decided upon later.

src/api -> all code that contains rest endpoints and serialisation/deserialisation code, error handling on api calls and no business logic.
src/game_engine -> all code that does the flow of the game, not individual data entities that it requires to work, those should go to src/models
src/models -> all the datastructs the game engine requires to run but no game flow logic
src/storage -> all code related to data persistence, repositories, and storage mechanisms for game elements by UUID

Premise: This document outlines the technical direction of the game development.

General goals
All game objects should be serialised and deserialised using JSON
All game objects should be identifiable using Uuids
The simulation part of the game should be turn based

Architectural pattern:

Frontend (UI Only) ←→ API Gateway ←→ Game Engine (Backend)
     ↑                    ↑                ↑
- Rendering only     - Validation      - All calculations
- User input         - Authentication  - Game state
- Visual effects     - Rate limiting   - Simulations
- State display      - Serialization   - Business logic

All code should have tests
All structs should have tests, in a separate mod identified by "name of the struct"_tests.
All data entities should be identifyable by a UUID.
All data entities should have as little logic as possible, traits should be implemented for data entities to introduce logic.
All data entities should be serialisable to JSON and back.
All rest calls should have tests
All end points need to be documented
The game should always be able to save its state

Code Quality Standards:
- **All phases and steps must be checked for compiler warnings before completion**
- **All phases and steps must pass `cargo clippy` with zero warnings in new code**
- **Run `cargo clippy --lib` and fix all warnings related to the current work**
- **Use `cargo clippy --fix --lib --allow-dirty` for automatic fixes where applicable**
- **Unused variables should be prefixed with underscore (_) if intentional**
- **Test-only imports should use `#[cfg(test)]` attribute**
- **Dead code intended for future use should be marked with `#[allow(dead_code)]`**
- **A phase or step is NOT complete until all warnings and clippy remarks are resolved**

Storage Layer Guidelines:
- All game elements are stored as JSON strings indexed by UUID
- Storage layer provides generic repository pattern for CRUD operations
- Storage implementations include in-memory (for testing/dev) and file-based (for persistence)
- Models should not directly depend on storage - use game_engine as intermediary
- Storage layer is responsible for serialization/deserialization of game entities
- Each entity type can have its own storage directory for file-based persistence

Add uuid for entity IDs
Add sqlx or diesel for persistence
Add tokio-tungstenite for WebSocket support
Add rand for procedural generation
Add petgraph for complex relationships

Make a rudimentary UI first focusing on function and not visual

Place only one struct in a file and add a mod tests to place creation, retrieve, update and delete tests in a tests mod inside the file where the struct resides.
