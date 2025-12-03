# Abyssal Architect – Phase 1

Simulation & Rules Engine – Design Contract

## 1. Purpose & Scope

The **Simulation & Rules Engine** is the foundation of Abyssal Architect.

It is responsible for:

* Representing the dungeon, rooms, traps, monsters, heroes, and waves.
* Simulating a wave of heroes moving through the dungeon and resolving:

  * Pathfinding and room entry.
  * Trap triggers and cooldowns.
  * Monster AI and combat.
  * Hero abilities and status effects.
* Producing a **deterministic event log** and final state that the UI can replay/visualize.
* Compiling to **WebAssembly** for use in the TypeScript frontend and to a native library for offline tools/tests.

It is **not** responsible for:

* Rendering or UI.
* Meta-progression, unlocks, or user accounts.
* Network or persistence.
* High-level “game flow” (menus, run selection, dailies).

If Phase 1 is correct and stable, later phases can iterate without constantly rewriting combat logic.

---

## 2. Tech Stack & Boundaries

**Primary implementation language:**

* **Rust** for core simulation logic.

**Targets:**

* **WASM module** consumed by the TypeScript client.
* **Native library / CLI** for:

  * Python balancing tools.
  * Automated tests / fuzzing.

**Public surface from this phase:**

* A narrow, versioned API (think “engine contract”) that later phases call:

```rust
// Pseudocode / shape only
fn simulate_wave(
    dungeon: DungeonState,
    wave: WaveConfig,
    seed: u64,
    max_ticks: u32,
) -> SimulationResult;
```

* All types exposed across the boundary are:

  * Serializable to JSON (or equivalent) for debugging.
  * Stable-ish across the project (versioned if needed).

No other modules are allowed to reach into the internals. All game logic runs via the engine API.

---

## 3. Responsibilities (in detail)

### 3.1 Domain Model

Define **core immutable concepts**:

* **DungeonState**

  * Rooms (graph structure, adjacency).
  * Room contents: traps, monsters, special tiles.
  * Dungeon “core” / heart node.
* **WaveConfig**

  * List of hero groups, entry spawn(s), and timings.
  * Difficulty tier and modifiers (e.g. “+25% HP, fire-resistant”).
* **Unit models**

  * `Hero`: HP, armor, speed, abilities, AI flags, tags (e.g. “rogue”, “healer”).
  * `Monster`: same, plus behavior flags (e.g. guard room, chase, ranged).
  * `Trap`: trigger conditions, damage/effects, cooldown/charges.
  * `StatusEffect`: poison, burn, slow, fear, etc.
* **Simulation config**

  * Tick duration (ms equivalent).
  * Movement rules (how far a unit goes per tick).
  * Targeting rules (who gets hit, priorities).

### 3.2 Simulation Loop

The engine must implement:

* **Deterministic tick-based loop:**

  * At each tick:

    * Update hero intents (movement/combat).
    * Resolve trap triggers and cooldowns.
    * Apply damage and status effects.
    * Remove dead heroes/monsters.
  * Stop when:

    * All heroes are dead, **or**
    * Dungeon core is destroyed, **or**
    * `max_ticks` exceeded (safety).

* **Pathfinding:**

  * Heroes choose paths based on:

    * Shortest distance to core, adjusted by:

      * Path weight modifiers.
      * “Scouting” traits (e.g., rogues avoid trap-heavy paths).
  * Deterministic even with equal-cost paths (e.g., tie breaks by room ID).

* **Event log emission:**

  * For each tick, record:

    * Movements.
    * Triggers (traps, abilities).
    * Damage events.
    * Status changes.
    * Deaths and core HP changes.
  * This event log is the *only* source of truth for the client to animate.

### 3.3 Determinism & Reproducibility

The engine must guarantee:

* Given:

  * Same `DungeonState`,
  * Same `WaveConfig`,
  * Same `seed`,
  * Same `max_ticks`,
* Then:

  * Simulation produces the **exact same `SimulationResult`** byte-for-byte.

Requirements:

* All randomness uses a **local RNG** seeded from input `seed`.
* No calls to system time, OS randomness, or non-deterministic APIs.
* No reliance on non-deterministic hash map iteration.

---

## 4. Non-Responsibilities (Hard Boundaries)

The Phase 1 engine:

* **Does NOT**:

  * Read/write files.
  * Talk to any network or backend.
  * Know about user profiles, unlocks, or daily seeds.
  * Apply monetization or difficulty scaling across runs.

* **Only knows**:

  * “I take an abstract dungeon + wave + seed, and I return a deterministic simulation result.”

Any attempt by later phases to sneak extra concerns into the engine should be rejected unless Phase 1 is explicitly extended.

---

## 5. Data & API Contract

### 5.1 High-Level Types (language-agnostic)

You can think of the engine’s main types like this (shape only):

```ts
// TS-facing shapes (mirrored in Rust with serde)
type DungeonState = {
  rooms: RoomState[];
  edges: { fromRoomId: string; toRoomId: string }[];
  coreRoomId: string;
};

type RoomState = {
  id: string;
  traps: TrapInstance[];
  monsters: MonsterInstance[];
  tags: string[]; // e.g., ["fire", "choke-point"]
};

type WaveConfig = {
  waveId: string;
  heroes: HeroSpawn[];
  modifiers: string[]; // e.g., ["elite", "fire_resist"]
};

type HeroSpawn = {
  heroTypeId: string;
  count: number;
  spawnRoomId: string;
  delayTicks: number;
};

type SimulationResult = {
  outcome: "heroes_win" | "dungeon_win" | "timeout";
  finalDungeon: DungeonState;
  finalHeroes: HeroInstance[];
  stats: SimulationStats;
  events: SimulationEvent[];
};
```

The **exact canonical definitions live in Rust**, auto-generated or mirrored into TypeScript via bindings.

### 5.2 Versioning

* The engine exposes a **semantic version**:

  * `engine_version: "1.0.0"` as part of `SimulationResult`.
* Any breaking changes to data shapes:

  * Bump MAJOR or MINOR.
  * Later phases must check `engine_version` when deserializing replay data.

---

## 6. Performance & Safety Requirements

### 6.1 Performance

At Phase 1 completion:

* For a **typical wave** (e.g., 50–100 heroes, 30–60 rooms, 50–100 entities total):

  * Simulation of a full wave should complete in **< 50 ms** in WASM on a mid-tier laptop.
* For tooling:

  * The native engine should be able to simulate **10k+ waves per minute** in batch mode on a dev machine (for balancing).

### 6.2 Safety & Limits

* Hard caps (configurable but enforced) on:

  * Max heroes per wave.
  * Max entities per room.
  * Max ticks per simulation.
* Engine must **fail fast** with a structured error if limits are exceeded, not hang or crash.

---

## 7. Testing & QA

### 7.1 Unit Tests (Rust)

Must cover at least:

* Pathfinding:

  * Single path, multiple equal paths, blocked paths.
* Combat:

  * Basic attack, damage, and death.
  * Multiple attackers.
* Traps:

  * Single-use, cooldown-based, AoE traps.
  * Line-of-sight / range checks.
* Status Effects:

  * Poison (damage over time).
  * Slow (movement modifier).
  * Buffs (damage/defense modifiers).

### 7.2 Property/Fuzz Tests

At least one fuzz/property test suite:

* Given random but valid:

  * Dungeons, waves, and seeds.
* Assert:

  * No panics or crashes.
  * Deterministic results for same seed & input.
  * Invariants (e.g., HP never negative, entity positions valid).

### 7.3 Golden Replay Tests

* Curate a small set of **golden scenarios**:

  * “Simple corridor”.
  * “Forked path with traps”.
  * “Elite wave with healer”.
* Store expected `SimulationResult` snapshots.
* Unit tests assert the current engine matches these snapshots byte-for-byte.

  * Purpose: catch regressions when you refactor rules.

### 7.4 WASM Integration Tests

* A minimal TypeScript harness that:

  * Loads the WASM module.
  * Calls `simulate_wave` with a variety of inputs.
  * Asserts:

    * Serialization/deserialization works.
    * Events list is sane and complete.

---

## 8. Observability & Debuggability

Even in Phase 1, this engine needs to be **debuggable by humans**.

Requirements:

* Ability to run the simulation in **step mode**:

  * `step_simulation(state, rng_state) -> (state', events_for_this_tick)`
* Ability to generate a **human-readable textual log** from `SimulationResult` for debugging:

  * e.g., `"T=23: Hero#4 triggered trap 'fire_glyph' in Room#12 – took 15 dmg (HP 35 → 20)"`.
* Simple CLI tool (Rust) to:

  * Load a JSON `DungeonState` and `WaveConfig` from disk.
  * Run a simulation with a seed.
  * Print summary stats and optionally full logs.

---

## 9. Dependencies & Interfaces With Later Phases

Phase 1 acts as a **platform** for other phases:

* **Phase 2 – Game Client & UI (TypeScript)**

  * Uses WASM engine:

    * Sends `DungeonState + WaveConfig + seed`.
    * Receives `SimulationResult` and animates using the event log.
  * Does not implement game logic; only uses engine output.

* **Phase 3 – Content & Balancing Tools (Python)**

  * Uses the native engine:

    * Batch-simulates many runs to evaluate difficulty.
    * Reads/writes JSON dungeon/wave definitions.

* **Phase 4 – Backend & Meta-Progression (Go)**

  * May run server-side validations:

    * Re-simulate submitted runs for daily leaderboards (same seed, same inputs).
    * Sanity-check that clients are not cheating.

Because of that, **stability of the Phase 1 API matters**. The fewer breaking changes after this phase, the better.

---

## 10. Allowed Technical Debt in Phase 1

To avoid gold-plating, these shortcuts are explicitly allowed:

* Monster/hero AI can start **simpler** than final vision:

  * e.g. basic “move towards core, attack nearest enemy” with a small set of abilities.
* Only a **subset** of planned status effects has to be implemented (e.g. poison, slow, stun).
* The event log format can be slightly verbose/inefficient:

  * Premature optimization is optional as long as it meets the performance bar.

However, these are **not allowed**:

* Hidden randomness not derived from the seed.
* Leaking implementation details into Phase 2 (e.g. UI-specific IDs).
* Mixing render concerns into event types (“draw this sprite” is Phase 2’s job).

---

## 11. Definition of Done (Checklist)

Phase 1 is **complete** when all of the following are true:

1. **Core model**

   * [ ] `DungeonState`, `WaveConfig`, and unit/trap models are defined in Rust.
   * [ ] They serialize/deserialize to a documented JSON schema.

2. **Simulation**

   * [ ] `simulate_wave` implemented with deterministic tick-based rules.
   * [ ] Hard safety limits on entities and ticks are enforced.

3. **Determinism**

   * [ ] Re-running the same inputs+seed yields identical results.
   * [ ] Property tests confirm determinism over many random scenarios.

4. **Tests**

   * [ ] Unit tests for pathfinding, combat, traps, and status effects.
   * [ ] Golden scenario tests pinned.
   * [ ] WASM integration tests passing.

5. **Tooling**

   * [ ] CLI tool exists to run simulations from JSON and print logs.
   * [ ] At least one script (Python or Rust) runs thousands of simulations for perf sanity.

6. **Docs**

   * [ ] Short engine reference: data shapes + main APIs.
   * [ ] “How to add a new trap/hero” dev doc (for future content work).

When this checklist is all ✅, Phase 2 (Game Client & UI) can proceed **without touching game rules**, and balancing/content work can piggyback on this engine.
