# Abyssal Architect – Phase 4

Backend & Meta-Progression Platform – Design Contract

---

## 1. Purpose & Scope

Phase 4 adds a **server-backed spine** to Abyssal Architect.

It is responsible for:

* **User identity & auth**

  * Guest accounts + simple tokens.
  * Stable `user_id` used across sessions/devices (optionally).
* **Meta-progression & profile state**

  * Persistent shards/xp/level.
  * Persistent content unlocks (traps, monsters, relics, starting layouts).
  * Persistent achievements/flags.
* **Run submission & validation**

  * Accepting run summaries from clients.
  * Verifying them by re-simulating with the Rust engine.
  * Using validated runs to grant progression and later to drive leaderboards.
* **Daily/weekly challenges**

  * Server-generated canonical seeds and rulesets.
  * Exposed via simple HTTP APIs.
* **Canonical content distribution**

  * Authoritative `content_pack` versions & hashes.
  * Letting clients know if they’re outdated.

Phase 4 is **not** responsible for:

* Fancy social features (friends, clans).
* Deep analytics dashboards (only basic logging/metrics).
* Live balance adjustment based on telemetry (Phase 6).
* Payment/monetization.

---

## 2. Dependencies & Boundaries

### 2.1 Depends On

* **Phase 1 – Engine**

  * Exposes native (non-WASM) API or CLI for:

    * `simulate_wave` and/or `simulate_run`.
  * Deterministic given same input & seed.
* **Phase 2 – Client**

  * Can play a full run locally.
  * Can serialize a run summary: choices, seed, outcome, score.
* **Phase 3 – Content**

  * Produces `content_pack.vX.json` with:

    * Stable IDs for all content.
    * `content_hash` and `version`.

### 2.2 Provides To

* **Client (TS)**

  * Auth tokens.
  * Profile state (meta currency, unlock IDs).
  * Daily/weekly challenge definitions.
  * Content manifests (which content pack version to use).
* **Later Phases**

  * Phase 5 (leaderboards, online UX) will piggyback on:

    * Run storage.
    * Challenge definitions.
    * Verification logic.
  * Phase 6 (telemetry/live ops) will reuse DB schema and events.

Backend must be **authoritative** for:

* Profile state.
* What’s considered a valid run for progression/challenges.

---

## 3. Tech Stack & Architecture

### 3.1 Main Stack

* **Go** (1.22+):

  * HTTP API server.
  * Business logic (auth, profiles, runs, challenges).
* **Database**

  * Relational DB (e.g. PostgreSQL).
  * Migration via standard migration tool.
* **Rust**

  * Engine compiled as:

    * Native library used via FFI *or*
    * Separate binary invoked by Go via subprocess.
* **TypeScript**

  * Client integration: API layer calling the Go backend.

### 3.2 Service Layout (Phase 4 Level)

Implement as a single Go **monolith** with clear internal modules:

* `auth`: guest auth, token issuance.
* `profile`: meta-progression, unlocks.
* `runs`: run submission, verification, storage.
* `challenges`: daily/weekly challenge generation & retrieval.
* `content`: manifest endpoint, serving which content pack version/hash is active.

Later phases can split this into microservices if necessary; Phase 4 does not.

---

## 4. Data Model (Server-Side)

Tables/entities (conceptual; adjust names in actual migrations):

### 4.1 Users

```text
users
------
id                (UUID / bigint)
created_at        (timestamp)
last_seen_at      (timestamp)
auth_type         (enum: guest, email, external_provider)
auth_identifier   (nullable string; e.g., device_id, email)
```

* Each device/guest gets a `user` created on first auth.
* `auth_identifier` can be a device-fingerprint or later email/SSO.

### 4.2 Profiles (Meta-Progression)

```text
profiles
--------
user_id                 (FK -> users.id, PK)
level                   (int)
xp                      (int)
shards                  (int)     # meta-currency
core_unlocked           (bool)    # simple global flag if needed
unlocks_json            (jsonb)   # normalized later if needed
achievements_json       (jsonb)
last_content_version    (string)
```

`unlocks_json` example structure:

```json
{
  "traps": ["trap_fire_glyph", "trap_spike_pit", ...],
  "monsters": ["monster_ember_guard", ...],
  "relics": ["relic_furnace_core"],
  "layouts": ["layout_basic_core"]
}
```

Server treats **content IDs** from the content pack as canonical.

### 4.3 Runs

```text
runs
----
id                    (UUID / bigint)
user_id               (FK -> users.id)
started_at            (timestamp)
finished_at           (timestamp)
content_version       (string)
engine_version        (string)
challenge_id          (nullable string)  # e.g., "daily_2025-01-12"
seed                  (bigint)
score                 (int)
outcome               (enum: dungeon_win, heroes_win, timeout)
validated             (bool)             # after re-simulation
validation_error      (nullable text)
payload_hash          (string)           # hash of submitted run payload
payload_json          (jsonb)            # full or partial run summary (for now)
meta_awarded          (bool)             # whether progression was applied from this run
```

The raw `payload_json` stores:

* Summary of picks, waves cleared, etc.
* Enough info to reconstruct run for verification (see section 6).

### 4.4 Challenges (Daily / Weekly)

```text
challenges
----------
id                    (string)  # e.g. "daily_2025-12-03"
type                  (enum: daily, weekly, event)
start_at              (timestamp)
end_at                (timestamp)
seed                  (bigint)
content_version       (string)
rules_json            (jsonb)   # modifiers, restrictions, scoring rules
```

Example `rules_json`:

```json
{
  "mutators": ["heroes_fire_resist_25", "traps_double_cooldown"],
  "banned_relics": ["relic_furnace_core"],
  "forced_relics": ["relic_cursed_seed"],
  "scoring": {
    "base_multiplier": 1.0,
    "bonus_per_tier": 50
  }
}
```

---

## 5. API Surface (v1)

All endpoints prefixed with `/v1`.

### 5.1 Auth

#### `POST /v1/auth/guest`

* **Purpose:** Create or resume a guest user.
* **Request body:**

```json
{
  "device_id": "optional-device-fingerprint-string"
}
```

* **Response:**

```json
{
  "user_id": "uuid-or-int",
  "token": "jwt-or-signed-token",
  "created": true  // true if new user, false if existing
}
```

Token used in `Authorization: Bearer <token>` header for subsequent calls.

### 5.2 Profile

#### `GET /v1/profile`

* **Auth:** required.

* **Purpose:** Fetch the user’s meta-progression state and allowed content.

* **Response:**

```json
{
  "user_id": "123",
  "level": 5,
  "xp": 230,
  "shards": 420,
  "unlocks": {
    "traps": ["trap_fire_glyph", "trap_spike_pit"],
    "monsters": ["monster_ember_guard"],
    "relics": ["relic_furnace_core"],
    "layouts": ["layout_basic_core"]
  },
  "achievements": ["first_blood", "tier3_clear"],
  "content_version": "1.0.0"
}
```

Client uses `unlocks` and `content_version` to filter/validate what content can appear in drafts.

### 5.3 Content Manifest

#### `GET /v1/content/manifest`

* **Purpose:** Tell client what content version/hash is current.

* **Response:**

```json
{
  "active_version": "1.0.0",
  "content_hash": "sha256:abc123...",
  "min_supported_client_version": "0.3.0",
  "download_url": "https://.../content_pack.v1.json"
}
```

Client compares its local content pack; if mismatch or outdated, it can:

* Download new pack.
* Or refuse to submit runs until updated.

### 5.4 Run Submission

#### `POST /v1/run/submit`

* **Auth:** required.

* **Purpose:** Submit a completed run for progression and (later) leaderboard.

* **Request body (shape):**

```json
{
  "content_version": "1.0.0",
  "engine_version": "1.0.0",
  "seed": 123456789,
  "challenge_id": "daily_2025-12-03",  // or null
  "score": 850,
  "outcome": "dungeon_win",
  "run_summary": {
    "tiers_cleared": 5,
    "waves_cleared": 23,
    "core_hp_remaining": 42,
    "draft_choices": [
      {
        "tier": 1,
        "choice_id": "draft_option_001",
        "picked_type": "trap",
        "picked_id": "trap_fire_glyph"
      }
      // ...
    ],
    "final_dungeon": {/* compressed dungeon state or ID references */},
    "stats": {
      "heroes_killed": 120,
      "damage_dealt_by_traps": { "trap_fire_glyph": 1200 },
      "damage_dealt_by_monsters": { "monster_ember_guard": 600 }
    }
  }
}
```

* **Server behavior:**

  1. Lookup `user_id` from token.
  2. Validate `content_version` is allowed.
  3. Store raw run record (unvalidated).
  4. Attempt **re-simulation** (see section 6) to verify:

     * Outcome matches.
     * Score within allowed tolerance (ideally exact).
  5. If valid:

     * Set `validated = true`.
     * Compute progression rewards (XP, shards, unlocks).
     * Update `profiles`.
  6. Return updated profile + summary.

* **Response (success):**

```json
{
  "run_id": "456",
  "validated": true,
  "rewards": {
    "xp_gained": 50,
    "shards_gained": 25,
    "new_unlocks": {
      "traps": ["trap_poison_cloud"],
      "relics": []
    }
  },
  "profile": {
    "level": 6,
    "xp": 280,
    "shards": 445,
    "unlocks": { /* ... */ }
  }
}
```

* **Response (suspected-cheat or mismatch):**

```json
{
  "run_id": "456",
  "validated": false,
  "reason": "mismatch_outcome_or_score",
  "profile": { /* unchanged profile */ }
}
```

Client can still show local “GG” but does not get server progression.

### 5.5 Challenges

#### `GET /v1/challenge/today`

* **Auth:** optional (but recommended).

* **Purpose:** Fetch today’s daily challenge.

* **Response:**

```json
{
  "id": "daily_2025-12-03",
  "type": "daily",
  "seed": 987654321,
  "content_version": "1.0.0",
  "rules": {
    "mutators": ["heroes_fire_resist_25", "traps_half_cooldown"],
    "forced_relics": ["relic_furnace_core"],
    "banned_traps": ["trap_poison_cloud"]
  }
}
```

Client uses this to configure a run and tag submissions correctly.

---

## 6. Run Verification Strategy

Core principle: **server trusts engine, not clients**.

### 6.1 Minimal Reconstruction Data

To re-simulate a run server-side, we need:

* **content_version** and its exact pack (server already has it).
* **engine_version** (for consistency).
* **seed** used for run-level RNG.
* **sequence of draft picks**:

  * Enough to deterministically reconstruct the dungeon state & available options.
* **challenge rules** (for challenge runs).

Approach:

* Client submits `draft_choices` as:

  * For each draft:

    * The list of offered option IDs (or a deterministic index) **and** which one was picked.
  * Or simply enough info for the server to re-run the draft RNG using the same content + seed.

* Server replays:

  * Same deterministic draft algorithm as client (using the same RNG seeded from `seed`).
  * Applies the same picks to build the final dungeon.
  * Uses engine to simulate all waves and calculate:

    * Score.
    * Outcome.

If the run flow (randomness, drafting) is fully deterministic, no additional data beyond picks and seed are needed.

### 6.2 Validation Rules

Run is considered **valid** if:

* Client’s `content_version` matches server’s active version.
* Engine re-sim using supplied seed & picks:

  * Produces **same outcome**.
  * Produces **same score** (or within clearly documented tolerance; ideally exact).
* Draft picks:

  * All picked content IDs are available based on user’s unlocks and rules (daily rules, etc.).

Invalid if:

* Any mismatch above.
* Content ID usage disallowed by challenge or unlock state.
* suspicious patterns (later phases may fingerprint).

Invalid runs:

* Stored with `validated = false` and `validation_error` set.
* Do **not** award XP/shards/unlocks.

---

## 7. Meta-Progression Design

### 7.1 XP & Level

* Each **validated run** awards:

  * `xp_gained = f(tiers_cleared, difficulty, challenge_type)`.
* Level thresholds:

  * e.g., `level_n_required_xp` table in content or config.
* Level-ups:

  * Unlock new content, grant shards, maybe visual cosmetics later.

### 7.2 Shards (Meta Currency)

* Earned proportional to performance:

  * e.g., score / 10, with caps.
* Spent on:

  * Unlocking specific traps/monsters/relics.
  * Unlocking alternate layouts or starting relics.

Phase 4 scope: **unlock logic** can be simple:

* Automatic unlocks at certain levels or achievements.
* Optional “spend shards to unlock” API kept client-side for now (server does the real update).

### 7.3 Unlock Rules

Configurable (driven by content pack flags):

* Each content item has `unlock_tier` and optional `unlock_requirements`:

```json
{
  "id": "trap_poison_cloud",
  "unlock_requirements": {
    "min_level": 5,
    "required_achievements": ["first_boss_kill"],
    "shard_cost": 100
  }
}
```

Server logic:

* On profile update (after run):

  * Compute which new items become eligible (based on new level/achievements).
  * Auto-unlock those with `shard_cost == 0`.
  * For shard-cost unlocks:

    * Optional explicit future endpoint: `POST /v1/unlock/purchase`.

Phase 4: it’s sufficient to support **automatic unlocks**; explicit purchases can be optional or stubbed.

---

## 8. Client Integration Flow

### 8.1 Session Bootstrap

1. Client starts, loads local `content_pack` (Phase 3).
2. Client calls `POST /v1/auth/guest` (or cached token).
3. Client calls `GET /v1/content/manifest`:

   * If `active_version != local.content_version` or `hash` mismatch:

     * Prompt update / restrict run submission until updated.
4. Client calls `GET /v1/profile`:

   * Get `unlocks` and current meta state.

### 8.2 Standard Run

1. Client starts local run (Phase 2 flow) using:

   * `unlocks` limitations when building draft options.
2. When run ends:

   * Client calls `POST /v1/run/submit` with run summary.
3. Backend validates and responds with updated profile + rewards.
4. Client updates its local profile cache.

### 8.3 Daily Challenge Run

1. Client calls `GET /v1/challenge/today`.
2. Client starts run with:

   * Provided `seed`, `rules`, `content_version`.
3. On completion, client submits run via `/v1/run/submit` with `challenge_id`.
4. Backend handles exactly like standard run, plus marking it as challenge-eligible.

---

## 9. Testing & QA

### 9.1 Unit Tests (Go)

Coverage:

* Auth:

  * Guest creation, token validation, invalid tokens rejection.
* Profile:

  * Level/xp updates.
  * Unlock computation: simple and edge cases.
* Runs:

  * Input validation for `run/submit`.
  * Reward computation (xp/shards).
* Challenges:

  * Correct challenge selection for given date/time.

### 9.2 Integration Tests

* Using a test DB + engine stub or real engine:

1. **Happy path:**

   * Create guest.
   * Fetch profile.
   * Submit a valid run payload (using a recorded known-good example).
   * Verify:

     * `runs.validated = true`.
     * `profiles` updated as expected.

2. **Invalid run:**

   * Send mismatching `score`/`outcome`.
   * Verify:

     * `runs.validated = false`.
     * No progression applied.

3. **Content mismatch:**

   * Submit run with stale `content_version`.
   * Verify run rejected or flagged.

### 9.3 Engine Integration

* At least one test that:

  * Uses the **real Rust engine binary/library** from Go.
  * Simulates a known scenario and checks outcome.

This ensures the FFI/subprocess integration isn’t broken.

---

## 10. Observability & Ops

Phase 4 must include:

* **Structured logging**:

  * For each run submission:

    * user_id, run_id, content_version, challenge_id, seed, validation_result.
* **Basic metrics** (via Prometheus/OpenTelemetry style):

  * Count of runs submitted / validated / rejected.
  * Counts of challenge runs per day.
  * Latency of `/v1/run/submit`.
* **Admin tooling (basic)**

  * CLI or small admin endpoint to:

    * List recent invalid runs with reasons.
    * Force-generate next N daily challenges.

---

## 11. Allowed Technical Debt in Phase 4

Allowed:

* Only **guest auth** in this phase; email/SSO later.
* Single Go service rather than micro-services.
* “Fire and forget” run verification:

  * If engine call is too slow, simple synchronous model is okay initially, with a hard timeout.

Not allowed:

* Progression without run validation (no “trust client blindly”).
* Skipping content version checks (server must know what pack was used).
* Returning generic errors without enough info to debug (seed, content version, user_id).

---

## 12. Definition of Done (Checklist)

Phase 4 is **complete** when:

1. **Core backend**

   * [ ] Go server runs with config (DB DSN, engine path, etc.).
   * [ ] Database migrations create `users`, `profiles`, `runs`, `challenges`.

2. **Auth & Profiles**

   * [ ] `POST /v1/auth/guest` implemented and tested.
   * [ ] `GET /v1/profile` returns correct state based on DB.
   * [ ] Profile unlock state respects content IDs and rules from the content pack.

3. **Content manifest**

   * [ ] `GET /v1/content/manifest` returns active content version/hash.
   * [ ] A simple process exists to update active content version (config or admin tool).

4. **Runs**

   * [ ] `POST /v1/run/submit` stores runs and attempts validation via engine.
   * [ ] Valid runs update XP, shards, and unlocks.
   * [ ] Invalid runs do not impact profile and are logged appropriately.

5. **Challenges**

   * [ ] `GET /v1/challenge/today` returns consistent daily challenge.
   * [ ] Backfill script or scheduler can create daily challenge rows.

6. **Client integration**

   * [ ] Client can:

     * Authenticate as guest.
     * Fetch profile.
     * Play local runs using unlock info.
     * Submit runs and see progression update.
   * [ ] Client handles “run rejected/unvalidated” gracefully.

7. **Tests & observability**

   * [ ] Unit and integration tests covering the main flows.
   * [ ] Logs and basic metrics are in place for run submissions and challenge retrieval.

Once all of that is ✅, you’ve got a server-backed meta layer that:

* Keeps progression and content IDs consistent.
* Validates runs via the Rust engine.
* Serves daily challenges.

From here, Phase 5 can focus on **UX polish, leaderboards, and richer online features** without having to redo the foundation.
