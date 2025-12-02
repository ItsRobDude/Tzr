This document outlines the technical specifications and communication protocols—the "Code Contract"—for the development of "Gravity Sling." This contract ensures interoperability between the TypeScript front-end, the Go real-time game server, and the API services (Python/Java).

This contract is presented in phases, mirroring the development plan, allowing for iterative implementation and testing.

### Shared Data Definitions

These fundamental structures will be used across all services and phases.

```
// Basic 2D vector representation (position or velocity)
interface Vector2D {
  x: number;
  y: number;
}

// Unique identifier for any entity in the game
type EntityID = string;
```

-----

### Phase 1: Connection and Basic Movement (WebSocket)

This phase focuses on establishing the connection, handling player inputs, and synchronizing basic movement. Communication occurs over WebSockets between the Client (TypeScript) and the Game Server (Go).

#### 1.1. Client-to-Server (C2S) Messages

**Message Type: `C2S_JOIN_REQUEST`**
Sent immediately upon connecting to the WebSocket.

```
{
  "type": "JOIN_REQUEST",
  "payload": {
    "nickname": "PlayerName",
    "authToken": "optional-jwt-token" // For authenticated sessions (See Phase 4)
  }
}
```

**Message Type: `C2S_INPUT`**
Sent frequently by the client (e.g., 30 times/sec) to update the player's intent.

```
{
  "type": "INPUT",
  "payload": {
    // Incremental number used for client-side prediction and server reconciliation
    "sequenceNumber": 1024,
    // Normalized vector indicating desired movement direction
    "movementDirection": { "x": 0.71, "y": -0.71 }
    // Beam activation will be added in Phase 2
  }
}
```

#### 1.2. Server-to-Client (S2C) Messages

**Message Type: `S2C_JOIN_ACCEPTED`**
Response to `JOIN_REQUEST`, providing initial configuration.

```
{
  "type": "JOIN_ACCEPTED",
  "payload": {
    "success": true,
    "assignedPlayerId": "player_abc123",
    "serverTickRate": 60, // Ticks per second
    "arenaConfig": {
      "width": 5000,
      "height": 5000
      // Obstacles will be added in Phase 3
    }
  }
}
```

**Message Type: `S2C_STATE_UPDATE`**
The primary synchronization packet sent every server tick.

```
{
  "type": "STATE_UPDATE",
  "payload": {
    "tick": 5000, // Current server tick
    // The sequenceNumber of the last input the server processed for THIS client
    "lastProcessedInput": 1024,
    "playerStates": [
      {
        "id": "player_abc123",
        "position": { "x": 500.0, "y": 500.0 },
        "velocity": { "x": 10.5, "y": -5.2 }
        // Mass and beam status will be added in Phase 2
      }
      // ... other nearby players
    ]
  }
}
```

-----

### Phase 2: Core Mechanics - Gravity Beam and Mass (WebSocket)

This phase introduces the gravity beam interaction and the collection of mass via Stardust.

#### 2.1. Client-to-Server (C2S) Updates

The input payload is updated to include the gravity beam state.

**Message Type: `C2S_INPUT` (Updated)**

```
{
  "type": "INPUT",
  "payload": {
    "sequenceNumber": 1025,
    "movementDirection": { "x": 0.71, "y": -0.71 },
    "beamActive": true // NEW: True if the activation button is held
  }
}
```

#### 2.2. Server-to-Client (S2C) Updates

The state update is expanded to include Stardust and the current mass/status of players.

**Message Type: `S2C_STATE_UPDATE` (Updated)**

```
{
  "type": "STATE_UPDATE",
  "payload": {
    "tick": 5060,
    "lastProcessedInput": 1025,
    "playerStates": [
      {
        "id": "player_abc123",
        "position": { "x": 510.0, "y": 495.0 },
        "velocity": { "x": 12.5, "y": -6.2 },
        "mass": 150.0,      // NEW
        "isBeaming": true   // NEW
      }
      // ...
    ],
    "stardustUpdates": [ // NEW: Updates for spawned or moved Stardust
      {
        "id": "dust_001",
        "position": { "x": 800.0, "y": 200.0 },
        "value": 5.0
      }
      // ...
    ],
    "removedEntityIds": ["dust_000"] // NEW: IDs of consumed Stardust or eliminated players
  }
}
```

-----

### Phase 3: Environment and Events (WebSocket)

This phase defines the arena obstacles (Asteroids) and the protocol for communicating significant game events.

#### 3.1. Server-to-Client (S2C) Updates

The initial configuration is updated to include static obstacles.

**Message Type: `S2C_JOIN_ACCEPTED` (Updated)**

```
{
  "type": "JOIN_ACCEPTED",
  "payload": {
    // ... previous fields
    "arenaConfig": {
      "width": 5000,
      "height": 5000,
      "obstacles": [ // NEW
        {
          "id": "ast_01",
          "type": "ASTEROID",
          "position": { "x": 1000.0, "y": 1000.0 },
          "radius": 150.0
        }
        // ...
      ]
    }
  }
}
```

**Message Type: `S2C_GAME_EVENT`**
Used for immediate, non-persistent events (e.g., sound effects, UI updates).

```
{
  "type": "GAME_EVENT",
  "payload": {
    "eventType": "PLAYER_ELIMINATED",
    "data": {
      "victimId": "player_abc123",
      "aggressorId": "player_xyz789", // Null if eliminated by the boundary
      "cause": "BOUNDARY_COLLISION"
    }
  }
}
```

**Message Type: `S2C_LEADERBOARD_UPDATE`**
Sent periodically (e.g., every 3 seconds) for the real-time in-game leaderboard.

```
{
  "type": "LEADERBOARD_UPDATE",
  "payload": {
    "topPlayers": [
      { "id": "player_xyz789", "name": "CometSmasher", "mass": 5000.0 }
      // ... top 5-10 players
    ]
  }
}
```

-----

### Phase 4: Persistence and API Contract (HTTP REST)

This contract defines the interaction with the API server (Python/Java) for managing persistent data, authentication, and matchmaking, using standard HTTP methods and JSON payloads.

#### 4.1. Server Discovery / Matchmaking

**Endpoint:** `GET /api/v1/servers/matchmake`
**Description:** Finds the most suitable game server (based on ping and capacity).
**Response (200 OK):**

```
{
  "host": "ws://game-server-host:7350",
  "lobbyId": "lobby-xyz-123"
}
```

#### 4.2. Authentication (Optional)

**Endpoint:** `POST /api/v1/auth/login`
**Description:** Authenticates a user and returns a JWT to be used in the WebSocket `JOIN_REQUEST`.
**Request:** `{ "username": "...", "password": "..." }`
**Response (200 OK):** `{ "token": "<JWT>", "userId": "..." }`

#### 4.3. Leaderboard Submission (Server-to-Server)

The Go Game Server submits statistics to the API server when a player is eliminated. This must be a secure server-to-server call.

**Endpoint:** `POST /api/v1/game/report_stats`
**Request:**

```
{
  "playerId": "player_abc123",
  "stats": {
    "kills": 5,
    "maxMassAchieved": 5000.0,
    "survivalTimeSeconds": 120
  }
}
```

**Response (202 Accepted):**

```
{
  "success": true,
  "message": "Stats received and queued for processing."
}
```

#### 4.4. Retrieving Global Leaderboard

The Client (TypeScript) requests persistent leaderboard data from the API server (Python/Java).

**Endpoint:** `GET /api/v1/leaderboard`
**Description:** Retrieves the top players based on a specific metric.
**Query Parameters:**

  * `metric` (string, required): The stat to rank by (e.g., `maxMass`, `totalKills`).
  * `timeframe` (string, optional): Filter results (e.g., `daily`, `weekly`, `allTime`. Default: `allTime`).
  * `limit` (int, optional): Number of results to return (Default: 50).

**Response (200 OK):**

```
{
  "metric": "maxMass",
  "timeframe": "allTime",
  "rankings": [
    {
      "rank": 1,
      "userId": "user_001",
      "nickname": "GravMaster",
      "score": 9500.50
    },
    {
      "rank": 2,
      "userId": "user_055",
      "nickname": "SlingShot",
      "score": 8900.00
    }
    // ...
  ]
}
```

#### 4.5. Retrieving Player Profile

The Client (TypeScript) requests aggregate statistics for a specific player.

**Endpoint:** `GET /api/v1/users/{userId}/stats`
**Description:** Fetches the lifetime statistics for the specified user.
**Response (200 OK):**

```
{
  "userId": "user_001",
  "nickname": "GravMaster",
  "stats": {
    "totalGamesPlayed": 1500,
    "totalKills": 4500,
    "lifetimeMaxMass": 9500.50,
    "averageSurvivalTimeSeconds": 85.2
  }
}
```

-----

### Phase 5: Serialization and Optimization

This phase outlines the strategy for serialization and optimizing network traffic, which is critical for a real-time .io game.

#### 5.1. HTTP API Serialization (Python/Java)

All communication with the HTTP API will use `Content-Type: application/json`.

#### 5.2. WebSocket Serialization Strategy (Go \<-\> TypeScript)

For initial development (Phases 1-3), JSON serialization over WebSockets is acceptable for rapid prototyping and ease of debugging.

**However, for production deployment, a binary protocol is required.** JSON is too verbose and slow to parse for high-frequency updates (up to 60 times/sec). We will utilize a schema-based binary protocol such as **Protocol Buffers (protobuf)** or **FlatBuffers**.

#### 5.3. Affected Messages

The following high-frequency messages MUST be converted to the binary protocol:

  * `C2S_INPUT`
  * `S2C_STATE_UPDATE`

The following infrequent messages can remain JSON:

  * `C2S_JOIN_REQUEST`
  * `S2C_JOIN_ACCEPTED`
  * `S2C_GAME_EVENT` (Unless extremely frequent)
  * `S2C_LEADERBOARD_UPDATE` (As this is typically sent every few seconds)

#### 5.4. Example Binary Schema (Protobuf)

This defines the contract for the binary data structure, replacing the JSON structure for the state update and input messages.

```protobuf
syntax = "proto3";

// Shared definition
message Vector2D {
  // Using float32 reduces payload size significantly compared to float64 (double)
  float x = 1;
  float y = 2;
}

// C2S_INPUT replacement
message ClientInput {
  uint32 sequenceNumber = 1;
  Vector2D movementDirection = 2;
  bool beamActive = 3;
}

// Component of S2C_STATE_UPDATE
message PlayerState {
  string id = 1; // Or use uint32/uint64 if IDs are numeric for further optimization
  Vector2D position = 2;
  Vector2D velocity = 3;
  float mass = 4;
  bool isBeaming = 5;
}

// Component of S2C_STATE_UPDATE
message StardustUpdate {
  string id = 1;
  Vector2D position = 2;
  float value = 3;
}

// S2C_STATE_UPDATE replacement
message GameSnapshot {
  uint32 tick = 1;
  uint32 lastProcessedInput = 2; // Specific to the client receiving this message
  repeated PlayerState playerStates = 3;
  repeated StardustUpdate stardustUpdates = 4;
  repeated string removedEntityIds = 5;
}
```

-----

### Phase 6: Error Handling and Conventions

#### 6.1. HTTP API Conventions and Status Codes

The API service will adhere to standard RESTful practices and HTTP status codes:

  * `200 OK`: Successful GET or synchronous POST/PUT.
  * `202 Accepted`: Request accepted for asynchronous processing (e.g., stat submission).
  * `400 Bad Request`: Malformed JSON or missing required parameters.
  * `401 Unauthorized`: Missing or invalid authentication token.
  * `404 Not Found`: Endpoint or resource (e.g., user profile) does not exist.
  * `500 Internal Server Error`: Unexpected error on the server.

All error responses (4xx and 5xx) will follow this standard JSON format:

```json
{
  "error": "ERROR_CODE_STRING",
  "message": "A human-readable description of the error."
}
```

#### 6.2. WebSocket Conventions (Go)

  * **Connection Management:** The client (TypeScript) is responsible for implementing a reconnection strategy with exponential backoff if the WebSocket connection drops unexpectedly. The server (Go) must temporarily hold the player's state. If a connection is not re-established within a defined timeout (e.g., 10 seconds), the player entity should be removed from the active game simulation, and their session terminated.
  * **Heartbeat/Ping-Pong:** To ensure the connection remains active and to detect network issues promptly, the server will implement standard WebSocket PING frames (e.g., every 15 seconds). The client must respond with PONG frames. Failure to respond within a timeout (e.g., 5 seconds) will result in the connection being terminated by the server.
  * **Rate Limiting:** The server must implement rate limiting on incoming `C2S_INPUT` messages. Clients should not send inputs significantly faster than the established `serverTickRate`. Clients exceeding this rate should be disconnected with code `4002`.
  * **Forceful Disconnection:** If the Game Server needs to disconnect a client, it will send a specific WebSocket close code and a reason string before terminating the connection. We will utilize the application-specific range (4000-4999) for custom errors:
      * `1000`: Normal Closure (e.g., the player chose to leave or the match ended).
      * `4001`: Invalid Authentication Token (encountered during `C2S_JOIN_REQUEST`).
      * `4002`: Protocol Violation (client sending malformed data, invalid inputs, or exceeding rate limits).
      * `4003`: Server Overloaded or Maintenance (client should attempt to connect to a different server instance).
      * `4004`: Kicked/Banned (administrative action).
      * `4005`: Duplicate Session (the same user connected from another client).

#### 6.3. Data Types and Constraints

To ensure consistency and prevent synchronization errors, the following constraints must be adhered to across all services.

  * **Floating Point Precision:**
      * For network transmission (WebSockets/Binary Protocol): `float32` (single precision) will be used for positions, velocities, and mass to minimize bandwidth.
      * For physics simulation (Go/TypeScript): `float64` (double precision) should be used internally for accuracy, then truncated to `float32` before transmission.
      * For persistent storage (Database/API): `float64` or fixed-point decimal types must be used.
  * **Integers:** Sequence numbers, tick counts, and numeric IDs should be `uint32` or `uint64`.
  * **Coordinates and Units:**
      * The coordinate system is Cartesian. The origin `(0,0)` will be defined in the `S2C_JOIN_ACCEPTED` configuration.
      * Time durations are in milliseconds unless otherwise specified.
      * Angles are in Radians.
  * **String Lengths:** Player nicknames must be sanitized, trimmed, and limited to 16 characters.

#### 6.4. Security Considerations

  * **Authoritative Server (Go):** The fundamental security principle is that the Game Server is the sole authority on the game state. The server must never trust client data regarding its position, mass, or status. It only accepts player *inputs* (intent).
  * **Input Validation (Go):** All inputs received must be validated. Crucially, the `movementDirection` vector must be normalized (or capped) on the server-side to prevent "speed hacking," even if the client claims it is already normalized.
  * **API Security (Python/Java):** All endpoints must implement appropriate authentication (JWT/OAuth) and authorization. Rate limiting and strict CORS policies must be enforced.
  * **Server-to-Server Communication:** Communication between the Go server and the API server (e.g., `report_stats`) must be authenticated using shared secrets, API keys, or mTLS.
  * **WSS/HTTPS:** In production, all communication must be encrypted using Secure WebSockets (WSS) and HTTPS.

#### 6.5. Versioning

  * **API Versioning:** The HTTP API will use URI versioning (e.g., `/api/v1/...`). Breaking changes require incrementing the version number.
  * **Game Protocol Versioning:** The `C2S_JOIN_REQUEST` must include a `protocolVersion` field. The server must reject clients that are too outdated to communicate using the current binary schema or game rules.

<!-- end list -->

```json
// Example addition to JOIN_REQUEST
{
  "type": "JOIN_REQUEST",
  "payload": {
    "nickname": "PlayerName",
    "protocolVersion": "1.0.0"
    // ... other fields
  }
}
```

-----

### Phase 7: Shared Physics Constants and Configuration

This phase defines the constants that govern the game's physics. For client-side prediction and reconciliation to function smoothly, these values must be identical on the Client (TypeScript) and the Server (Go).

#### 7.1. Simulation Constants

```typescript
// Shared Physics Configuration (must match Server implementation)
const SIMULATION_CONFIG = {
  // The rate at which the simulation updates
  TICK_RATE: 60,
  // Time elapsed per tick (delta time), calculated as 1 / TICK_RATE
  TICK_DT: 1 / 60,

  // Movement mechanics
  BASE_ACCELERATION: 500.0,
  // Friction or drag applied every tick to slow down entities
  LINEAR_DAMPING: 0.98,
  // Factor determining how mass affects top speed and acceleration
  MASS_INERTIA_FACTOR: 0.05,

  // Gravity Beam mechanics
  BEAM_MAX_RANGE: 300.0,
  // Base force applied by the beam
  BEAM_BASE_FORCE: 1500.0,
  // How much the relative mass difference amplifies the beam force
  BEAM_MASS_AMPLIFIER: 1.2,

  // Arena boundaries (Defined during JOIN_ACCEPTED, but defaults used for reference)
  DEFAULT_ARENA_WIDTH: 5000.0,
  DEFAULT_ARENA_HEIGHT: 5000.0
};
```

-----

### Phase 8: Environment Variables and Service Configuration

This section defines the expected configuration parameters required by the different services for deployment, managed via environment variables.

#### 8.1. Game Server (Go) Configuration

| Variable Name | Description | Example Value |
| :--- | :--- | :--- |
| `GS_PORT` | The port the WebSocket server listens on. | `7350` |
| `GS_TICK_RATE` | Updates per second (must match SIMULATION\_CONFIG). | `60` |
| `GS_API_ENDPOINT` | URL of the API server for internal communication. | `http://api-service:8000` |
| `GS_API_SECRET` | Shared secret for securing server-to-server calls. | `(Secure_Token)` |
| `GS_MAX_PLAYERS` | Maximum concurrent players this instance supports. | `200` |


#### 8.2. API Server (Python/Java) Configuration

This service handles persistence, authentication, and matchmaking APIs.

| Variable Name | Description | Example Value |
| :--- | :--- | :--- |
| `API_PORT` | The port the HTTP server listens on. | `8000` |
| `CORS_ALLOWED_ORIGINS`| Comma-separated list of origins allowed for API access. | `https://play.gravitysling.io,http://localhost:3000` |
| `DB_HOST` | Database hostname (e.g., PostgreSQL). | `postgres-db` |
| `DB_PORT` | Database port. | `5432` |
| `DB_USER` | Database username. | `gravity_user` |
| `DB_PASSWORD` | Database password. | `(Secure_Password)` |
| `DB_NAME` | Database name. | `gravity_sling_prod` |
| `JWT_SECRET_KEY` | Secret key used for signing authentication tokens. Must be strong and random. | `(Secure_Token)` |
| `GS_API_SECRET` | Shared secret for validating incoming calls from the Game Server (must match Game Server's config). | `(Secure_Token)` |

#### 8.3. Front-End (TypeScript) Configuration

The client configuration is typically injected during the build process (e.g., using Vite or Webpack environment variables).

| Configuration Key | Description | Example Value (Dev) | Example Value (Prod) |
| :--- | :--- | :--- | :--- |
| `API_BASE_URL` | The base URL for making HTTP REST calls. | `http://localhost:8000/api/v1` | `https://api.gravitysling.io/api/v1` |
| `MATCHMAKER_ENDPOINT`| The specific path used to find an available game server via the API. | `/servers/matchmake` | `/servers/matchmake` |
| `ENABLE_DEBUG_MODE` | Flag to enable verbose logging and physics visualization tools. | `true` | `false` |
| `INTERPOLATION_BUFFER_MS` | The time (in ms) the client renders "in the past" to smooth out updates from the server. Crucial for smooth visuals. | `100` | `100` |
| `WS_OVERRIDE_URL` | (Optional) Directly connect to a specific WebSocket server, bypassing the matchmaker. | `ws://localhost:7350` | N/A |

*(Note: The client does not configure the primary WebSocket connection string directly; it receives the specific WebSocket URL by calling the matchmaker endpoint defined in Phase 4.1).*

-----

### Phase 9: Logging, Monitoring, and Observability

To ensure stability and diagnose issues across the distributed architecture, standardized logging and metric collection are mandatory.

#### 9.1. Structured Logging

All backend services (Go, Python/Java) must output logs in a structured, machine-readable format, specifically **JSON Lines (NDJSON)**, to `stdout`. This allows centralized logging systems (e.g., ELK stack, Grafana Loki, Datadog) to ingest them efficiently.

**Required Log Fields:**

| Field | Type | Description |
| :--- | :--- | :--- |
| `@timestamp` | ISO 8601 | The exact time the event occurred. |
| `level` | String | Severity (`DEBUG`, `INFO`, `WARN`, `ERROR`, `FATAL`). |
| `service` | String | The name of the application (e.g., `game-server`, `api-service`). |
| `message` | String | A human-readable description of the event. |
| `correlation_id`| String | (Optional) A UUID tracing a request across multiple services. |
| `context` | Object | Context-specific data (e.g., player ID, error stack trace). |

**Example Log Entry (Game Server):**

```json
{"@timestamp":"2025-12-01T22:05:42Z", "level":"INFO", "service":"game-server", "message":"Player disconnected", "context":{"playerId":"player_abc123", "reason":"Timeout"}}
```

#### 9.2. Key Performance Metrics

Services must expose metrics, ideally in a format compatible with Prometheus.

**Game Server (Go) Metrics:**

  * `gs_active_connections`: Gauge. Total number of currently connected WebSockets.
  * `gs_tick_duration_ms`: Histogram. The time taken to process a single game tick. This is the most critical health metric and must remain below `1/TICK_RATE` (e.g., \<16.6ms for 60 TPS).
  * `gs_cpu_usage_percent`: Gauge. Current CPU utilization of the server process.
  * `gs_memory_usage_bytes`: Gauge. Current memory usage.
  * `gs_network_traffic_bytes_total`: Counter. Total incoming and outgoing bandwidth.

**API Server (Python/Java) Metrics:**

  * `api_http_requests_total`: Counter. Total number of HTTP requests, dimensioned by endpoint and status code.
  * `api_http_request_duration_seconds`: Histogram. Latency of HTTP requests.
  * `api_database_query_duration_seconds`: Histogram. Latency of database queries.
  * `api_authentication_failures_total`: Counter. Total number of failed login attempts.

#### 9.3. Client-Side Monitoring (TypeScript)

The client application must integrate an error monitoring service (e.g., Sentry). Additionally, the client should track and report the following performance indicators:

  * Frames Per Second (FPS) stability.
  * WebSocket latency (Ping).
  * Frequency and magnitude of client-side prediction errors (desyncs/reconciliations).
  * Unhandled JavaScript exceptions and stack traces.

-----

### Phase 10: Architecture and Deployment Strategy

#### 10.1. High-Level Architecture

The system will be deployed as a set of microservices, managed via container orchestration (e.g., Kubernetes).

1.  **Client (TypeScript):** Served via CDN/static hosting.
2.  **Load Balancer / API Gateway:** Entry point for traffic, handling SSL termination and routing.
3.  **Game Servers (Go):** Multiple instances running concurrently. These are **stateful** and manage the real-time game simulation. The Load Balancer must support "sticky sessions" for WebSocket routing, ensuring a client stays connected to the same server instance during a game.
4.  **API Server (Python/Java):** Handles HTTP requests. This service is **stateless** and can be scaled horizontally.
5.  **Database:** Persistent storage (e.g., PostgreSQL) for user data and statistics.

#### 10.2. Scalability Strategy

The primary challenge is scaling the stateful Game Servers. This will be achieved by deploying more instances and using the Matchmaking API to intelligently distribute players across available, healthy instances based on capacity and geographic location. The API server can be scaled horizontally as needed, and the database will utilize connection pooling and read replicas.

-----

### Phase 11: Testing Requirements

Adherence to the code contract must be verified through automated testing.

#### 11.1. Unit Testing

  * **Go:** Comprehensive unit tests for the physics engine, input validation, spatial hashing algorithms, and state management logic.
  * **TypeScript:** Unit tests for the rendering logic, client-side prediction and interpolation algorithms, and UI components.
  * **Python/Java:** Unit tests for authentication logic, leaderboard calculations, and database interactions.

#### 11.2. Integration and Contract Testing

  * **Protocol Compliance:** Tests that simulate the interaction between the Client and the Game Server, verifying that the binary protocol (Phase 5) is correctly implemented by both sides. This ensures the server correctly interprets client inputs and generates valid snapshots.
  * **API Contract Testing:** Automated tests that verify the API Server adheres to the RESTful conventions, HTTP status codes, error formats, and response payloads defined in Phases 4 and 6.

#### 11.3. End-to-End (E2E) and Load Testing

  * **Bot Simulation/Load Testing:** A dedicated test runner that spawns a Game Server instance and connects multiple headless clients (bots) to simulate a high-load scenario. This verifies physics interactions under stress, latency mitigation effectiveness, and the server's stability when nearing maximum capacity.
  * **Full User Journey:** E2E tests that simulate the complete user flow: authenticating with the API, requesting a match via the matchmaker, receiving a WebSocket URL, and successfully joining the game instance.
