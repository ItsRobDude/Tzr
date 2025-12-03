## 1. Purpose & Scope

The **Game Client & Run Loop** is the player-facing shell of Abyssal Architect.

It is responsible for:

* Presenting a playable **single-run experience**:

  * Drafting rooms/traps/monsters.
  * Running hero waves.
  * Showing results and run-end summary.
* Integrating with the **Phase 1 Simulation Engine (Rust → WASM)** to:

  * Build input state (dungeon + wave configs).
  * Invoke simulations deterministically.
  * Reconstruct visual playback from the event log.
* Managing **client-side run state**:

  * Current dungeon layout.
  * Player resources within a run (essence, gold, etc.).
  * Draft pools and choices.
* Providing a minimal but solid **UI framework** that later phases can extend:

  * Main menu.
  * Run screen.
  * Basic settings.

It is **not** responsible for:

* Long-term meta-progression (account, unlocks, dailies, etc.).
* Backend persistence or user accounts.
* Matchmaking, leaderboards, cloud validation.
* Deep analytics or telemetry dashboards (only local debug logging in this phase).

Phase 2 delivers “you can play a full run, locally, with real rules and a decent UI.”

---

## 2. Dependencies & Boundaries

### 2.1 Depends on Phase 1 – Simulation & Rules Engine

Phase 2 assumes Phase 1 is implemented and exposed as:

* A WASM module with at least:

  * `simulate_wave(dungeon: DungeonState, wave: WaveConfig, seed: u64, max_ticks: u32): SimulationResult`
  * Optional step-wise API for debugging: `step_simulation(...)`.
* A **documented JSON schema** for:

  * `DungeonState`
  * `WaveConfig`
  * `SimulationResult` (including event log)

Phase 2:

* **Reads** from Phase 1 only via the WASM API.
* Must not re‑encode game mechanics in TypeScript beyond:

  * Lightweight **client-only derived values** (e.g., simple tooltip text).
  * Animation timing interpolation that does not alter game logic.

### 2.2 Boundaries with Later Phases

Phase 2 will be used by:

* **Phase 3 – Content & Balancing Tools (Python)**

  * Reuses UI representations for debugging and visualizing tool output (e.g., replaying simulated waves).
* **Phase 4 – Backend & Meta-Progression (Go)**

  * Replaces local stub services with real API calls for:

    * Profile loading.
    * Meta-currency / unlocks.
    * Daily seeds or shared runs.

In this phase:

* Backend calls are either **absent** or **purely stubbed** (local mock services).
* Save data is **local only** (browser storage / local JSON), with a scope limited to:

  * Settings.
  * Optional last-run replay.

---

## 3. Tech Stack & Architecture

### 3.1 Languages & Libraries

* **TypeScript** as primary language.
* **React** (or similar) for UI composition.
* **PixiJS** or **Phaser** for the 2D game canvas:

  * Single main canvas for dungeon and combat.
  * React handles panels/menus; canvas handles visuals.

Rust (Phase 1) is consumed via:

* WASM bundle loaded at app startup.
* Thin TS wrapper providing:

  * Type-safe bindings for `simulate_wave`.
  * Conversion between JS objects and WASM memory.

### 3.2 High-Level Architecture

Packages / modules (conceptual):

* `ui/app`: app shell, routing, top-level providers.
* `ui/game`:

  * `RunController`: finite-state machine for a run.
  * `DungeonView`: canvas rendering of rooms and entities.
  * `DraftPanel`: draft choices, tooltips, selection.
  * `WavePlayback`: handles animations from event log.
* `ui/state`:

  * `RunState`: current dungeon, deck, resources, run RNG seed.
  * `Config`: difficulty, settings, debug options.
* `engine/wasm`:

  * WASM loader + binding functions.
  * Guards to ensure engine is ready before use.
* `ui/debug`:

  * Dev overlay for event logs, tick stepping, seed display.

---

## 4. Game Flow & Screens (Phase 2 Scope)

### 4.1 Screens

Phase 2 must implement:

1. **Main Menu**

   * Buttons:

     * `Start Run`
     * `Settings`
     * `Quit` (or browser-appropriate behavior).
   * Shows build version and engine version (`engine_version` from Phase 1).

2. **Run Screen**

   * Main layout:

     * **Left**: Dungeon canvas (rooms, edges, heroes, monsters, traps).
     * **Right**: Panels:

       * Current resources (essence, gold, shards for this run).
       * Upcoming waves preview.
       * Draft choices (when applicable).
       * Tooltip/inspect panel.

3. **Run End Screen**

   * Shows:

     * Outcome (dungeon survives vs falls).
     * Tier reached.
     * Heroes killed / waves cleared.
     * Simple score.
   * Buttons:

     * `Restart Run (new seed)`
     * `Return to Main Menu`

4. **Settings Modal**

   * Basic options:

     * Simulation speed (1x / 2x / 4x).
     * Toggle damage numbers.
     * Accessibility options (text size / colorblind-friendly mode).
   * No account/meta config yet.

### 4.2 Run Loop State Machine

Conceptual finite states:

* `idle` (in menu)
* `run_initializing`
* `drafting`
* `wave_running`
* `wave_results`
* `run_over` (win/lose)

Transitions:

* `idle` → `run_initializing` when `Start Run` is pressed.
* `run_initializing` → `drafting` (initial draft / base choices).
* `drafting`:

  * Player picks exactly one draft option.
  * Then → `wave_running`.
* `wave_running`:

  * Call `simulate_wave` via WASM.
  * Animate based on event log.
  * Once playback done → `wave_results`.
* `wave_results`:

  * Update run stats, resources.
  * If win and more waves:

    * → `drafting`.
  * If dungeon core destroyed or final tier cleared:

    * → `run_over`.
* `run_over` → `idle` or → `run_initializing` (restart).

State machine must be:

* Explicitly modeled (e.g., with a reducer or small FSM library).
* Serializable in a dev/debug way (for debugging and replay).

---

## 5. Internal Run State & Data Model (Client Side)

### 5.1 RunState Shape (TS-side)

A minimal but explicit TS type:

```ts
type RunState = {
  seed: number;
  tier: number;
  dungeon: DungeonState;          // mirrors engine schema
  upcomingWaves: WaveConfig[];    // at least current + 1 future
  resources: {
    essence: number;
    gold: number;
  };
  draftPool: DraftOption[];       // current choices
  history: WaveHistoryEntry[];    // previous waves' summaries
};
```

Where:

* `DraftOption`:

  * A normalized union type:

    * AddRoom
    * AddTrap
    * AddMonster
    * GlobalModifier
* `WaveHistoryEntry`:

  * Seed used, wave index, outcome summary, and a handle to optional replay data (see below).

### 5.2 Replay Storage (Local)

Replay details:

* For the **most recent run**, Phase 2 may store:

  * `DungeonState` at wave start.
  * `WaveConfig`.
  * `SimulationResult.event[]`.

Storage:

* Local only:

  * Browser: `localStorage` or `IndexedDB`.
* Retention:

  * At most the last full run.
  * This is for debugging and QA; it is not a stable format guarantee for future.

---

## 6. WASM / Engine Integration

### 6.1 Loading & Initialization

Requirements:

* WASM module is loaded **once** at startup via a dedicated `EngineProvider`.
* Until WASM is ready:

  * Main menu can load.
  * `Start Run` must be disabled or show a “loading engine” status.
* Any errors during WASM load:

  * Surfaced with a clear user error + dev log.
  * Do not crash the entire UI; allow reload.

### 6.2 Calls to Engine

All engine invocations go through a **thin wrapper**:

```ts
interface EngineAPI {
  simulateWave(
    dungeon: DungeonState,
    wave: WaveConfig,
    seed: number,
    maxTicks: number
  ): Promise<SimulationResult>;
}
```

Design constraints:

* No direct access to WASM memory from UI code.
* No UI-specific assumptions inside the engine:

  * Event names and IDs should be generic (“UnitMoved”, “TrapTriggered”), and UI maps these to visuals.

### 6.3 Error Handling

If `simulateWave` errors:

* Show a non-intrusive error dialog.
* Log detailed diagnostic info to console (seed, tier, wave index).
* Allow:

  * Retry (with same inputs).
  * Abort run and return to main menu.

---

## 7. Visual Design & Interaction (Phase 2-Level)

### 7.1 Dungeon Rendering

Minimum visual requirements:

* Rooms:

  * Rendered as nodes (rectangles or tiles).
  * Show connections/edges between rooms.
* Contents:

  * Traps and monsters represented as **icons/small sprites** overlayed in the room.
* Heroes:

  * Simple shapes or icons that move between rooms along edges.
* Core:

  * Visually distinct central room; health indicated by bar or halo.

Interactions:

* Hover room → show tooltip:

  * Room name and tags.
  * Contents with summary stats (for now, static from content pack).
* Click room → pin details in a side panel.

### 7.2 Draft UI

Draft panel must:

* Display exactly 3 choices (configurable later).
* For each choice:

  * Name.
  * Short description.
  * Tags (“Fire”, “Control”, “Scaling”).
  * Impact preview (e.g., “+1 trap slot in room X”, “add poison aura”).
* Require explicit confirmation:

  * Click a card → highlight.
  * Press `Confirm` → apply choice and advance.

No complex card animations required in Phase 2; basic hover/selection feedback is enough.

### 7.3 Wave Playback

Playback requirements:

* Progression:

  * Animations follow event log tick order.
* Controls:

  * Pause / Resume.
  * Speed toggle (1x / 2x / 4x).
  * “Skip to results” button that:

    * Skips animations.
    * Immediately applies final state and shows `WaveResults`.
* Feedback:

  * Minimal hit FX:

    * Flash room border or unit sprite on damage.
  * Optional damage numbers toggle (off by default for clutter).

---

## 8. Testing & QA

### 8.1 Unit Tests

Coverage:

* **State machine**:

  * All transitions for run states.
  * Edge cases:

    * Trying to start a run before engine ready.
    * Ensuring no illegal transitions (e.g., triggering draft during `wave_running`).
* **Draft logic**:

  * Correct application of a DraftOption to a `DungeonState`.
  * Refusal to apply drafts in the wrong state.
* **Engine wrapper**:

  * If WASM returns a failure, wrapper produces typed errors.
  * Correct translation between TS structs and engine schema.

### 8.2 Integration Tests

Using a headless browser test runner (Playwright/Cypress/etc.):

* **Smoke flow**:

  * Launch app.
  * Start run.
  * Take at least 2 drafting steps.
  * Run a wave and reach `WaveResults`.
  * Finish run (lose or win).
* **Determinism from client perspective**:

  * With a fixed initial seed and choices:

    * The same event log structure is produced across test runs (within reason; full deep equality can be a later phase).

### 8.3 Visual / UX Tests

* Snapshot tests for:

  * Main menu layout.
  * Run screen layout with empty dungeon.
  * Draft panel rendering with a fixed stub draft pool.

Manual QA checklist:

* Resize browser window:

  * Layout remains usable at common resolutions (1080p, laptop).
* Performance:

  * A typical wave with 50–100 entities animates smoothly at 1x and 2x on mid-tier hardware.

---

## 9. Observability & Debugging Hooks

Even though no real backend exists yet, Phase 2 must:

* Provide an optional **debug overlay** (dev-mode only) that shows:

  * Current `RunState` summary (seed, tier, heroes alive, etc.).
  * Last N simulation events in text form.
* Log:

  * Engine version and build hash at startup.
  * Seeds and tier/wave indexes on simulation calls.
* Provide a “Copy debug info” button in error dialog that copies:

  * Engine version.
  * Client build version.
  * Current state snapshot (seed, tier, wave index).

This sets up clean debugging and later bug reports.

---

## 10. Allowed Technical Debt in Phase 2

Allowed:

* Basic, non-final UI theme and placeholder art, as long as:

  * Layout is stable.
  * Components are structured for later re-skinning.
* Using a simple global state solution (e.g., single React context) instead of fully modular store, provided:

  * Boundaries to engine calls and run state are clearly encapsulated.
* Minimal animation polish:

  * No tweening libraries required yet; simple linear animations are okay.

**Not allowed:**

* Implementing any combat rules in TypeScript that diverge from the engine.
* Hard-coding any meta-progression or backend assumptions into UI (e.g., user IDs, accounts).
* Hiding engine errors behind generic “something went wrong” without useful debug info.

---

## 11. Definition of Done (Checklist)

Phase 2 is **complete** when:

1. **Engine integration**

   * [ ] WASM module loads reliably in the client.
   * [ ] `simulateWave` is wrapped in a TS API with type-safe inputs/outputs.
   * [ ] Errors from the engine are surfaced with clear messages and debug info.

2. **Run loop**

   * [ ] State machine for the run is implemented and tested.
   * [ ] Player can:

     * Start a fresh run.
     * Make draft choices between waves.
     * Run multiple waves in sequence.
   * [ ] Runs end correctly with a clear result screen.

3. **UI**

   * [ ] Main menu, run screen, run-end screen, and settings modal exist and are navigable.
   * [ ] Dungeon canvas shows rooms, connections, heroes, traps, monsters at a basic visual level.
   * [ ] Draft choices are clearly presented and selectable.

4. **Playback**

   * [ ] Event log from `SimulationResult` drives animations.
   * [ ] Playback controls (pause, speed, skip) are working.
   * [ ] Typical waves run smoothly at 1x speed on target hardware.

5. **Persistence (local only)**

   * [ ] Optional last-run replay is stored locally and can be replayed from the main menu (or debug screen).
   * [ ] Basic settings (speed, accessibility toggles) persist across reloads.

6. **Tests**

   * [ ] Unit tests for state machine, draft application, and engine wrapper.
   * [ ] At least one end-to-end integration test validates “start → play → finish run”.
   * [ ] Snapshot tests for key UI states.

7. **Docs**

   * [ ] Short developer-oriented document:

     * How to wire a new screen into the app shell.
     * How to add a new DraftOption type in the UI.
   * [ ] README section explaining how to run the client in dev mode and what Phase 2 guarantees.

Once all of these are ✅, you have a **playable, local-only Abyssal Architect** client that’s solid enough for real iteration, while leaving meta, backend, and live ops for later phases.

