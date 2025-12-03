# Abyssal Architect – Phase 5

Online Features, UX & Polish – Design Contract

---

## 1. Purpose & Scope

Phase 5 takes Abyssal Architect from “solid core game with backend” to a **release‑ready product**:

* Adds **online features** beyond simple progression:

  * Leaderboards (daily, challenge-based, and global).
  * Ghost/replay support for top runs (server-backed).
* Delivers a **real UX pass**:

  * Visual identity, consistent UI, onboarding, readability.
  * Basic accessibility options.
* Tightens **performance and stability**:

  * Stable 60fps target on mid-tier hardware.
  * Good memory behavior and smoother animations.
* Adds basic **error/crash reporting and client diagnostics**.

Phase 5 is **not** responsible for:

* Deep telemetry-driven balance/AB testing (Phase 6).
* Monetization.
* Big social systems (friends, chat, guilds).

If Phase 5 is “done”, you can reasonably ship an Early Access build.

---

## 2. Dependencies & Boundaries

### 2.1 Depends On

* **Phase 1 – Engine**: stable deterministic sim; used both client-side (WASM) and server-side.
* **Phase 2 – Client**: fully playable local runs, event-driven UI.
* **Phase 3 – Content**: content-driven cards/traps/heroes with packs and tools.
* **Phase 4 – Backend & Meta-Progression**:

  * User auth + profile + unlocks.
  * Run submission + validation.
  * Challenges (daily/weekly) and content manifest.

Phase 5 is **not allowed** to break deterministic behavior or the run submission contract.

### 2.2 Provides To

* **Phase 6 – Live Ops / Telemetry**:

  * Leaderboard schema and APIs.
  * Client crash/error reporting pipeline.
  * Performance metrics (where to hook more detailed telemetry).

---

## 3. Tech Stack & Architecture

No stack changes, just using it harder:

* **Client:** TypeScript + React + Pixi/Phaser.
* **Backend:** Go + PostgreSQL + Rust engine binary/lib for validation.
* **Tools:** Python + Rust for balance/batch sims (unchanged).
* Optional: small CDN / object store to serve replays/content packs efficiently.

---

## 4. Feature Set (Phase 5 Scope)

### 4.1 Online Features

1. **Leaderboards**

   * Daily Challenge Leaderboard:

     * Scope: 1 per `challenge_id` (e.g., `daily_2025-12-03`).
     * Metric: `score` (primary), `finished_at` (tie-break).
   * Global “All-Time” Board:

     * Best runs across all non-challenge runs.
     * Or per “mode” (standard vs hardcore, if you add it later).
   * Per-user view:

     * Show your best rank for recent dailies.
     * Show personal best scores, even if not globally high.

2. **Ghost/Replays**

   * For leaderboard entries, allow:

     * **“Watch Run”** → client pulls replay data, reconstructs run, and re-simulates via engine.
   * Replays are generated client-side using:

     * Run’s `seed`.
     * Content version.
     * Recorded draft choices & challenge rules.

3. **Seed Sharing / Run Codes**

   * For standard runs:

     * Expose a shareable “Run Code”:

       * Contains seed + a small set of flags (e.g., difficulty, modifiers).
     * Allow “Start Run from Code” entry in client:

       * Client configures run identically (no progression reward unless validated by server normally).

### 4.2 UX / UI & Visual Polish

1. **Visual identity**

   * Consistent color palette, typography, and UI components.
   * Cohesive aesthetic around “abyssal machine/dungeon architect” theme.
   * Replace placeholder UI with deliberate layout and art (still MS Paint–capable resolution/style).

2. **Clarity & Information Presentation**

   * Improved tooltips:

     * Traps/monsters: stats, tags, synergies (e.g., “combos with Poison tags”).
   * Deck/Loadout Inspect:

     * Panel showing all drafted traps, monsters, relics.
   * Wave preview:

     * Compact view of upcoming wave composition and basic traits.

3. **Onboarding & Tutorials**

   * First-run guided tutorial:

     * Highlight draft panel, show how to inspect a room, etc.
   * Optional “Help” section explaining:

     * Synergy tags.
     * How scoring works.
     * What daily challenges mean.

4. **Accessibility & Settings**

   * Text scaling (small, medium, large).
   * Colorblind-friendly palette toggle.
   * Reduced-effects mode (less flashing, less motion).
   * Keybindings for common actions (pause, speed toggle, skip wave).

### 4.3 Performance & Stability

1. **Rendering and GC behavior**

   * Reduce React re-renders; isolate canvas from heavy React churn.
   * Object pooling in Pixi/Phaser for hero/monster/trap sprites.
   * Avoid per-frame allocations in animation loops.

2. **Simulation performance**

   * Ensure WASM sim calls stay well under budget for typical waves.
   * If needed, pre-simulate full wave and just play back events (already there via event log).

3. **Responsiveness**

   * Main thread frame time target: ≤ 16ms on mid-tier hardware at 1080p.
   * Fast UI transitions between screens.

### 4.4 Error Handling / Crash Reporting

* Client:

  * Global React error boundary.
  * Wrap `simulateWave` calls with try/catch and surface friendly errors.
* Backend:

  * Structured error responses for all APIs.
* Reporting:

  * Optional `POST /v1/client-event` to log:

    * Client errors.
    * Serious UI failures (crash, failed WASM init).
  * Minimal but enough for debugging real-world issues.

---

## 5. New / Extended Data Model

### 5.1 Leaderboards

New table:

```text
leaderboard_entries
-------------------
id                 (UUID / bigint)
leaderboard_scope  (string)   # e.g. "daily:2025-12-03", "global:standard"
user_id            (FK -> users.id)
run_id             (FK -> runs.id)
score              (int)
rank_cache         (int)      # denormalized; recalculated periodically
created_at         (timestamp)
```

Behavior:

* `leaderboard_scope` defines the board:

  * Daily: `daily:<challenge_id>` (`daily:2025-12-03`).
  * Global Standard: `global:standard`.
  * Global Hardcore: `global:hardcore` (if you add modes).
* On run submission success:

  * If run is eligible for one or more leaderboard scopes:

    * Insert/update leaderboard entry:

      * One best entry per user per scope.

### 5.2 Replays (Optional Table vs Derived)

You already have enough to reconstruct runs from `runs` table (`payload_json`, seed, picks). Phase 5 can:

* **Option A (simpler):** No extra table.

  * Use `runs` row as replay data.
* **Option B (explicit):** `replays` table if you later want replay pruning, compression, etc.

For Phase 5, Option A is sufficient.

### 5.3 Client Event Logs (Optional)

```text
client_events
-------------
id                 (UUID / bigint)
user_id            (nullable FK -> users.id)
client_version     (string)
content_version    (string)
platform           (string)  # "web", "android", etc.
event_type         (string)  # "client_error", "panic", "perf_warning"
payload_json       (jsonb)
created_at         (timestamp)
```

Useful for serious crash reports and impossible states.

---

## 6. Backend API Additions (v1)

### 6.1 Leaderboards

#### `GET /v1/leaderboard`

**Query params:**

* `scope`: string, required. Examples:

  * `scope=daily:2025-12-03`
  * `scope=global:standard`
* `limit`: optional, default 50.
* `offset`: optional, for pagination.

**Response:**

```json
{
  "scope": "daily:2025-12-03",
  "entries": [
    {
      "rank": 1,
      "user_id": "123",
      "display_name": "Player123",     // may be anonymous / generated if no name
      "score": 1420,
      "run_id": "456"
    },
    {
      "rank": 2,
      "user_id": "789",
      "display_name": "Player789",
      "score": 1405,
      "run_id": "457"
    }
  ],
  "your_best": {
    "rank": 25,
    "score": 980,
    "run_id": "999"
  }
}
```

### 6.2 Run Replay Fetch

#### `GET /v1/run/{run_id}/replay`

**Purpose:** Provide enough data for client to reconstruct run for ghost playback.

**Response:**

```json
{
  "run_id": "456",
  "user_id": "123",
  "content_version": "1.0.0",
  "engine_version": "1.0.0",
  "seed": 123456789,
  "challenge_id": "daily_2025-12-03",
  "run_summary": {
    "draft_choices": [ ... ],     // same structure as submit
    "rules": { /* challenge rules */ }
  }
}
```

Client:

* Validates content version matches local; otherwise either:

  * Refuses to play (simple).
  * Or loads that content pack (more advanced, can be Phase 6+).

### 6.3 Client Events

#### `POST /v1/client-event`

**Purpose:** Collect serious client-side issues.

**Request:**

```json
{
  "event_type": "client_error",
  "client_version": "0.5.0",
  "content_version": "1.0.0",
  "platform": "web",
  "payload": {
    "message": "Uncaught TypeError in WavePlayback",
    "stack": "stack trace string",
    "state": {
      "screen": "run",
      "tier": 3
    }
  }
}
```

Backend:

* Rate-limit per IP/user.
* Store as `client_events` row.
* Don’t break gameplay on failure.

### 6.4 Profile Settings (Optional but recommended)

#### `GET /v1/profile/settings` / `PUT /v1/profile/settings`

**Purpose:** Sync cross-device UX / accessibility settings.

Settings example:

```json
{
  "text_scale": "large",
  "colorblind_mode": "protanopia",
  "reduced_effects": true,
  "default_speed": "2x"
}
```

Client uses this at startup; still caches settings locally.

---

## 7. Client Work – UX, UI & Integrations

### 7.1 Leaderboard Screen(s)

New screens/panels:

1. **Daily Challenge Leaderboard**

   * Accessible from main menu (“Daily & Leaderboards”).

   * Shows:

     * Today’s challenge summary at top.
     * Top N entries with:

       * Rank, score, approximate build summary (e.g., icon grid for traps/relics).
     * “Your Best” row pinned.

   * Each entry:

     * Has a “Watch” button → calls `/run/{run_id}/replay` and starts ghost playback.

2. **Global Leaderboard**

   * Separate tab in the same UI.
   * Filter: mode (standard/hardcore) if modes exist.

### 7.2 Run Summary & End Screen Improvements

On run end:

* Rich summary:

  * Damage breakdown by trap/monster.
  * Relics used and their contributions.
  * Peak wave or “most dangerous wave”.
* Buttons:

  * `Submit Score` (if for some reason auto-submit fails).
  * `View Leaderboard` (for associated scope if challenge).

### 7.3 Ghost Playback Mode

When viewing a replay:

* Show **“GHOST RUN”** clearly (to avoid confusion with live play).
* UI differences:

  * No draft choices (just spectate).
  * Timeline scrubber:

    * Play, pause, jump to wave.
* Implementation:

  * Build the same run state from replay data and call engine locally via WASM.
  * Use event log playback like a normal run, but **no user input**.

### 7.4 Visual & UX Polish

Changes:

* Centralized theming system:

  * Colors, typography, button styles, card frames.
  * Use a theme provider so future tweaks are cheap.
* Tooltips:

  * Delay and positioning tuned for usability.
  * Show core stats, tags, synergy hints (derived from content pack).
* Animations:

  * Polished but simple: fades, slides, subtle scale on hover/selection.
  * Configurable via “reduced effects” setting.

### 7.5 Accessibility & Settings

Implement in client:

* Settings modal with:

  * Text size slider or discrete options.
  * Colorblind mode toggle (adjusts color variables).
  * Reduced effects toggle (turns off screen flashes, heavy motion).
  * Default sim speed.
* Persist:

  * Locally (e.g., localStorage).
  * Optionally server-side via `/v1/profile/settings`.

---

## 8. Performance & Stability Work

### 8.1 Frontend Performance

Targets:

* 60fps for typical waves at 1080p on mid-tier hardware.
* Smooth UI with minimal jank during heavy waves.

Work items:

* Pixi/Phaser:

  * Use sprite pools instead of constantly creating/destroying objects.
  * Avoid re-creating textures or filters mid-run.
* React:

  * Keep game canvas in a memoized component that rarely re-renders.
  * Use derived selectors for state, avoid passing giant state trees down.

### 8.2 Memory & Leaks

* Ensure:

  * Removing a run / ending a replay tears down references to event logs and sprites.
  * No accumulation of old WASM sim results.
* Tools:

  * Use browser dev tools to watch heap growth between runs.

### 8.3 Backend Performance

* Leaderboards:

  * Index `leaderboard_entries` on `(leaderboard_scope, score DESC, created_at ASC)`.
  * Use window functions (`ROW_NUMBER()`) or materialized ranking if necessary.
* Run verification:

  * Keep engine invocations within a strict timeout.
  * If needed, offload long verifications to a job queue (Phase 6), but Phase 5 can be synchronous with reasonable limits.

---

## 9. Testing & QA

### 9.1 Backend

Unit/integration tests for:

* `GET /v1/leaderboard`:

  * Correct sorting and pagination.
  * Correct “your_best” computation.
* Leaderboard insertion/update logic:

  * Only one best entry per user per scope.
* `GET /v1/run/{id}/replay`:

  * Only for validated runs.
  * Matches previously submitted summary.
* `POST /v1/client-event`:

  * Accepts valid payloads.
  * Properly rate-limited.

### 9.2 Client

Automated tests:

* Leaderboard UI:

  * Loads entries, highlights your best.
  * Clicking “Watch” enters ghost mode without crashing.
* Settings:

  * Text scaling and colorblind mode apply correctly.
  * Settings persist across reload.
* Tutorial:

  * First run triggers tutorial; subsequent runs can skip.

Manual QA:

* Play through:

  * Tutorial.
  * Several standard runs.
  * Several daily challenge runs.
  * Submissions and leaderboard views.
* Check:

  * Performance during heavy waves.
  * Error handling when server is offline.

---

## 10. Observability

By end of Phase 5:

* Backend:

  * Metrics:

    * Runs submitted per minute.
    * Leaderboard read requests.
    * Replay fetch requests.
  * Logs include:

    * Leaderboard scope, user_id, run_id for writes.
    * Replay fetch success/fail reasons.

* Client:

  * Error boundary that:

    * Shows friendly message to player.
    * Sends client-event for unhandled exceptions in production builds.

---

## 11. Allowed Technical Debt in Phase 5

Allowed:

* Leaderboards limited to:

  * Daily challenge scope and one global scope initially.
* Replays:

  * Only for recent dailies (e.g., last 7 days) to keep DB small.
* Ghost playback:

  * No mid-wave scrubbing at first; wave-level jumps only.

Not allowed:

* Awarding leaderboard positions for **unvalidated** runs.
* Ghost playback based on **client-side-only state** that cannot be reconstructed from run data.
* Crashes in client with no visible feedback (must be caught by error boundary).

---

## 12. Definition of Done (Checklist)

Phase 5 is **done** when:

1. **Online features**

   * [ ] Leaderboards exist for at least:

     * Daily challenges.
     * One global mode.
   * [ ] Valid runs automatically update leaderboard entries.
   * [ ] `/v1/leaderboard` and `/v1/run/{id}/replay` are implemented and tested.

2. **Replays**

   * [ ] Client can request a replay for a leaderboard entry.
   * [ ] Client reconstructs the run and plays it back deterministically.
   * [ ] Ghost mode is clearly labeled and input-disabled.

3. **UX & visual polish**

   * [ ] Theming system implemented.
   * [ ] Draft, dungeon, and run summary UIs are visually consistent and reasonably polished.
   * [ ] Onboarding/tutorial experience guides new players through at least one run.

4. **Accessibility & settings**

   * [ ] Text scaling options work.
   * [ ] Colorblind/reduced-effects modes work.
   * [ ] Settings persist across sessions (local and optionally via backend).

5. **Performance & stability**

   * [ ] Typical waves run smoothly at 1x/2x on target hardware.
   * [ ] No significant memory leaks across multiple runs/replays in test.
   * [ ] Backend leaderboard queries perform adequately under test load.

6. **Error handling & reporting**

   * [ ] Global error boundary in client.
   * [ ] Serious client errors generate `client-event` logs in backend.
   * [ ] Run submission/leaderboard/replay errors show understandable messages to the player.

When those are all ✅, you’re in “this can actually be launched” territory.
