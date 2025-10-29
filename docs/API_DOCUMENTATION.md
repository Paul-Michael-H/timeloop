# Timeloop API Documentation

## Overview

The Timeloop API provides RESTful endpoints for managing game definitions, game sessions, and character progression. All endpoints use JSON for request/response bodies.

**Base URL**: `http://localhost:3000`

**Content-Type**: `application/json` for all POST/PUT requests

---

## Authentication

Currently, no authentication is required. Authentication will be added in a future update.

---

## Error Responses

All endpoints may return the following error responses:

### Standard Error Format
```json
{
  "error": "Error message describing what went wrong"
}
```

### HTTP Status Codes
- `200 OK` - Request succeeded
- `201 Created` - Resource created successfully
- `204 No Content` - Resource deleted successfully
- `400 Bad Request` - Invalid input or validation error
- `404 Not Found` - Resource not found
- `409 Conflict` - Duplicate resource (e.g., duplicate name)
- `500 Internal Server Error` - Server error

---

## Health Check

### Check Server Health
**GET** `/health`

Returns the health status of the server.

#### Response
- **Status**: `200 OK`
- **Body**: Empty

#### Example
```bash
curl http://localhost:3000/health
```

---

## Attribute Definitions

Attributes represent character statistics like Strength, Agility, Intelligence, etc.

### Create Attribute
**POST** `/api/definitions/attributes`

Creates a new attribute definition.

#### Request Body
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Strength",
  "description": "Physical power and muscle capability",
  "category": "Physical",
  "base_value": 10,
  "min_value": 1,
  "max_value": 100,
  "training_difficulty": 100,
  "icon": null
}
```

#### Fields
- `id` (string, UUID) - Unique identifier (generated client-side)
- `name` (string, required) - Unique name for the attribute
- `description` (string, required) - Description of what the attribute represents
- `category` (enum, required) - One of: "Physical", "Mental", "Social", "Spiritual"
- `base_value` (number, required) - Starting value (must be between min and max)
- `min_value` (number, required) - Minimum allowed value
- `max_value` (number, required) - Maximum allowed value (must be > min_value)
- `training_difficulty` (number, required) - Difficulty percentage (1-100)
- `icon` (string, optional) - Icon identifier or path

#### Response
- **Status**: `201 Created`
- **Body**: The created attribute (same structure as request)

#### Example
```bash
curl -X POST http://localhost:3000/api/definitions/attributes \
  -H "Content-Type: application/json" \
  -d '{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Strength",
    "description": "Physical power",
    "category": "Physical",
    "base_value": 10,
    "min_value": 1,
    "max_value": 100,
    "training_difficulty": 100,
    "icon": null
  }'
```

#### Error Responses
- `400 Bad Request` - Validation error
  - Empty name
  - Empty description
  - Invalid range (min >= max)
  - Base value out of range
  - Training difficulty = 0
- `409 Conflict` - Attribute with this name already exists (case-insensitive)

---

### Get Attribute
**GET** `/api/definitions/attributes/:id`

Retrieves a specific attribute by ID.

#### Parameters
- `id` (path parameter) - UUID of the attribute

#### Response
- **Status**: `200 OK`
- **Body**: Attribute object

#### Example
```bash
curl http://localhost:3000/api/definitions/attributes/550e8400-e29b-41d4-a716-446655440000
```

#### Error Responses
- `404 Not Found` - Attribute with this ID does not exist

---

### Update Attribute
**PUT** `/api/definitions/attributes/:id`

Updates an existing attribute.

#### Parameters
- `id` (path parameter) - UUID of the attribute to update

#### Request Body
Same structure as Create Attribute. The `id` in the body must match the `id` in the URL.

#### Response
- **Status**: `200 OK`
- **Body**: The updated attribute

#### Example
```bash
curl -X PUT http://localhost:3000/api/definitions/attributes/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/json" \
  -d '{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Strength",
    "description": "Updated description",
    "category": "Physical",
    "base_value": 15,
    "min_value": 1,
    "max_value": 100,
    "training_difficulty": 100,
    "icon": null
  }'
```

#### Error Responses
- `400 Bad Request` - Validation error or ID mismatch
- `404 Not Found` - Attribute not found
- `409 Conflict` - Name conflicts with another attribute

---

### Delete Attribute
**DELETE** `/api/definitions/attributes/:id`

Deletes an attribute definition.

#### Parameters
- `id` (path parameter) - UUID of the attribute to delete

#### Response
- **Status**: `204 No Content`
- **Body**: Empty

#### Example
```bash
curl -X DELETE http://localhost:3000/api/definitions/attributes/550e8400-e29b-41d4-a716-446655440000
```

#### Error Responses
- `404 Not Found` - Attribute not found

---

### Search/List Attributes
**GET** `/api/definitions/attributes`

Searches or lists all attributes. If query parameter is provided, returns matching attributes. Otherwise, returns all attributes.

#### Query Parameters
- `q` (optional) - Search query (case-insensitive substring match on name)

#### Response
- **Status**: `200 OK`
- **Body**: Array of attribute objects

#### Examples

List all attributes:
```bash
curl http://localhost:3000/api/definitions/attributes
```

Search for attributes:
```bash
curl http://localhost:3000/api/definitions/attributes?q=strength
```

#### Response Example
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Strength",
    "description": "Physical power",
    "category": "Physical",
    "base_value": 10,
    "min_value": 1,
    "max_value": 100,
    "training_difficulty": 100,
    "icon": null
  },
  {
    "id": "650e8400-e29b-41d4-a716-446655440001",
    "name": "Physical Endurance",
    "description": "Stamina and resilience",
    "category": "Physical",
    "base_value": 10,
    "min_value": 1,
    "max_value": 100,
    "training_difficulty": 120,
    "icon": null
  }
]
```

---

## Game Session Management

### Create New Game
**POST** `/api/game/new`

Creates a new game session with a character.

#### Request Body
```json
{
  "character_name": "Aldric the Wise",
  "starting_age": 18
}
```

#### Response
- **Status**: `200 OK`
- **Body**: Game state with character details

#### Example
```bash
curl -X POST http://localhost:3000/api/game/new \
  -H "Content-Type: application/json" \
  -d '{"character_name": "Aldric", "starting_age": 18}'
```

---

### Get Current Game
**GET** `/api/game/current`

Retrieves the current active game session.

#### Response
- **Status**: `200 OK`
- **Body**: Current game state

#### Error Responses
- `404 Not Found` - No active game session

#### Example
```bash
curl http://localhost:3000/api/game/current
```

---

### Advance Tick
**POST** `/api/game/tick`

Advances the game by one tick (processes training, aging, etc.).

#### Response
- **Status**: `200 OK`
- **Body**: Updated game state

#### Error Responses
- `404 Not Found` - No active game session

#### Example
```bash
curl -X POST http://localhost:3000/api/game/tick
```

---

### Set Training
**POST** `/api/game/training`

Sets which attribute the character is training.

#### Request Body
```json
{
  "attribute_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

#### Response
- **Status**: `200 OK`
- **Body**: Updated game state

#### Error Responses
- `404 Not Found` - No active game or invalid attribute ID

#### Example
```bash
curl -X POST http://localhost:3000/api/game/training \
  -H "Content-Type: application/json" \
  -d '{"attribute_id": "550e8400-e29b-41d4-a716-446655440000"}'
```

---

### Save Game
**PUT** `/api/game/save`

Saves the current game session to disk.

#### Response
- **Status**: `200 OK`
- **Body**: Success message

#### Error Responses
- `404 Not Found` - No active game session

#### Example
```bash
curl -X PUT http://localhost:3000/api/game/save
```

---

### List Saves
**GET** `/api/game/saves`

Lists all available save files.

#### Response
- **Status**: `200 OK`
- **Body**: Array of save file names

#### Example
```bash
curl http://localhost:3000/api/game/saves
```

Response:
```json
[
  "aldric_2025-10-29.json",
  "gandalf_2025-10-28.json"
]
```

---

## Rate Limiting

Currently, no rate limiting is implemented. This will be added in a future update.

---

## Versioning

API version: `v1` (implicit)

Future versions will use URL prefixing: `/api/v2/...`

---

## Client Libraries

### Rust Client
The API client is included in the codebase:

```rust
use timeloop::editor::api_client::EditorApiClient;

let client = EditorApiClient::new("http://localhost:3000");
let attributes = client.list_attributes().await?;
```

See `src/editor/api_client.rs` for full implementation.

---

## Development

### Running the Server
```bash
cargo run --bin timeloop-server
```

Server starts on `http://localhost:3000`

### Testing the API
Run integration tests:
```bash
cargo test --test api_integration_tests
```

---

## Support

For issues or questions:
- Open an issue on GitHub
- Check `docs/plans/ARCHITECTURE_REFACTOR.md` for architecture details
