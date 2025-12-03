# Abyssal Architect – Phase 6

Live Ops, Telemetry & Expansion – Design Contract

---

## 1. Purpose & Scope

Phase 6 is about **running the game as a live product**:

* Seeing what players actually do.
* Keeping balance healthy and interesting.
* Shipping content and patches without breaking saves/runs.
* Running limited-time events and experiments in a controlled way.

It is responsible for:

* **Telemetry & analytics pipeline**

  * Structured gameplay events and metrics.
  * Storage and basic querying/aggregation.
* **Balance & content iteration loop**

  * Use real player data + engine sims to guide nerfs/buffs/content additions.
  * Patch safely with versioning.
* **Live events & seasons**

  * Time-limited mutators, event rules, and reward tracks.
* **Experimentation**

  * Basic A/B (or A/B/C) experiments on tuning and UX.
* **Release management**

  * Clear versioning for client, engine, content, backend.
  * Rollout/rollback paths.

It is **not** responsible for:

* Monetization (you can bolt that on top later if needed).
* Deep social systems (clans, chat, etc.).
* Huge BI infrastructure (Snowflake, fancy dashboards) – we’ll keep it sane and focused.

---

## 2. Dependencies & Boundaries

### 2.1 Depends On

* **Phase 1 – Engine**

  * Deterministic simulation used for both production runs and offline balance sims.
* **Phase 2 – Client**

  * Playable experience with clean state machine and event-driven UI.
* **Phase 3 – Content**

  * Content packs with version & hash.
* **Phase 4 – Backend & Meta-Progression**

  * Auth, profiles, runs, challenges, content manifest.
* **Phase 5 – Online Features & UX**

  * Leaderboards, replays, client error reporting, polished UI.

### 2.2 Provides To

* Game designers / devs:

  * A stable way to ship updates and know what they did.
* Engine/content:

  * Feedback loop based on real data.
* Future “Phase 7+” (if any):

  * Monetization and deeper systems can lean on this telemetry and event framework.

---

## 3. Tech Stack

No new major tech; we add a few pieces:

* **Backend:** Go + PostgreSQL (existing), with new tables for telemetry, experiments, events.
* **Analytics & Offline Tools:** Python + Rust engine for analysis & simulation.
* **Client:** TypeScript; emits telemetry events, reads event/experiment configs.
* Optional:

  * Stream pipeline (Kafka/Kinesis/whatever) if you outgrow DB-first, but Phase 6 assumes **DB-first telemetry** is fine.

---

## 4. Live Ops Feature Set

### 4.1 Telemetry & Analytics

Goals:

* Understand:

  * Run outcomes, win rates per tier.
  * Pick/usage rates and performance for traps/monsters/relics.
  * Daily challenge participation and difficulty.
  * Basic engagement: sessions, retention signals.

Core idea: **structured gameplay events** sent from client to backend, piggybacked on existing run submission when possible.

### 4.2 Balance & Content Iteration Loop

* Regular cycle:

  1. Pull data from telemetry and runs tables.
  2. Run Python/Rust tools to:

     * Identify outliers (OP/UP content, difficulty spikes).
     * Propose parameter changes.
  3. Test suggested changes via engine sim sweeps.
  4. Ship updated content pack and, if needed, engine tweaks.
  5. Track impact with new telemetry.

### 4.3 Events & Seasons

* Event types:

  * **Short events** (3–7 days): special mutators and reward boosts.
  * **Seasons** (multi-week): themed content packs, modifiers, and progression tracks.
* Each event/season:

  * Has ID, start/end times, set of mutators, eligible challenges, and reward rules (XP/shards multipliers, cosmetics later).

### 4.4 Experiments (A/B Testing)

* Ability to:

  * Randomly assign users to variants.
  * Change:

    * Drop rates / draft weights.
    * Minor tuning values (e.g., +10% HP, lower trap cooldown).
    * UI changes (e.g., tutorial variant, new draft layout).
* Track metrics per variant:

  * Win rate, retention proxies, score distributions.

### 4.5 Release Management

* Clear versioning for:

  * Client (`client_version`).
  * Engine (`engine_version`).
  * Content (`content_version` & `content_hash`).
  * Backend (`api_version` – implied by deployment).
* Rules:

  * Which combinations are allowed.
  * How to gradually roll out and roll back:

    * Content updates.
    * Engine changes.

---

## 5. Data Model Extensions

### 5.1 Telemetry Events

New table for structured events:

```text
telemetry_events
----------------
id               (UUID / bigint)
user_id          (nullable FK -> users.id)
session_id       (string)   # client-generated per app session
event_type       (string)   # "run_start", "run_end", "draft_pick", ...
event_time       (timestamp)
client_version   (string)
content_version  (string)
platform         (string)   # "web", "android", "desktop"
payload_json     (jsonb)    # event-specific fields
```

We’ll keep event schema compact but useful.

### 5.2 Event/Season Definitions

```text
live_events
-----------
id               (string)  # "event_fire_festival_01"
type             (enum: event, season)
start_at         (timestamp)
end_at           (timestamp)
content_version  (string)   # required or "*" for all
rules_json       (jsonb)    # mutators, reward multipliers, cosmetics flags
```

`rules_json` example:

```json
{
  "mutators": ["global_fire_damage_boost_20"],
  "challenge_bonus_multiplier": 1.5,
  "additional_unlocked_tags": ["fire"],
  "leaderboard_scopes": ["season:fire_festival_01"]
}
```

### 5.3 Experiments

```text
experiments
-----------
id             (string)   # "exp_draft_weights_v1"
name           (string)
description    (text)
start_at       (timestamp)
end_at         (timestamp)
status         (enum: draft, running, paused, finished)
config_json    (jsonb)    # variants & parameters
```

Example `config_json`:

```json
{
  "variants": [
    {
      "id": "control",
      "weight": 1,
      "parameters": {}          // baseline
    },
    {
      "id": "more_relics",
      "weight": 1,
      "parameters": {
        "draft_relic_weight_multiplier": 1.5
      }
    }
  ]
}
```

And per-user assignment:

```text
experiment_assignments
----------------------
user_id       (FK -> users.id)
experiment_id (FK -> experiments.id)
variant_id    (string)
assigned_at   (timestamp)
```

### 5.4 Extra Fields on Existing Tables

* `runs`:

  * Add `event_id` (nullable) and `experiment_variant_snapshot` (jsonb) to know what event/experiment context a run was played under.
* `profiles`:

  * Might track `current_season_id` for seasonal progression (if you add a battle-pass style track later).

---

## 6. Telemetry Event Taxonomy

Keep it tight and high-signal. Core events:

1. **Session events**

   * `session_start`
   * `session_end`
   * Payloads: platform, client_version, content_version.

2. **Run lifecycle**

   * `run_start`

     * `run_id`, `seed`, `challenge_id`, `event_id`, `experiment_variants`.
   * `run_end`

     * `run_id`, `outcome`, `score`, `tiers_cleared`, `waves_cleared`.

3. **Draft & build**

   * `draft_offered`

     * `run_id`, `tier`, `options` (ids).
   * `draft_pick`

     * `run_id`, `tier`, `picked_id`, `picked_type`.
   * `relic_pick`, `layout_pick` – if you have those distinct.

4. **Difficulty signals**

   * `wave_fail`

     * `run_id`, `tier`, `wave_idx`, `core_hp_at_death`.
   * `wave_clear`

     * `run_id`, `tier`, `wave_idx`, `heroes_remaining`.

5. **Engagement / progression**

   * `profile_level_up`

     * `old_level`, `new_level`.
   * `unlock_obtained`

     * `content_type`, `content_id`.

Many of these are derivable from runs, but emitting them explicitly makes analysis simpler and decoupled.

---

## 7. Backend API Additions

### 7.1 Telemetry Ingestion

#### `POST /v1/telemetry/batch`

* **Auth:** optional; attach token if you have it.
* **Purpose:** Ingest a **batch** of events to reduce chattiness.

**Request:**

```json
{
  "session_id": "abc-123",
  "client_version": "0.6.0",
  "content_version": "1.1.0",
  "platform": "web",
  "events": [
    {
      "event_type": "session_start",
      "event_time": "2025-12-03T10:00:00Z",
      "payload": { }
    },
    {
      "event_type": "run_start",
      "event_time": "2025-12-03T10:02:00Z",
      "payload": {
        "run_id": "local-temp-id-or-hash",
        "seed": 123456789,
        "challenge_id": "daily_2025-12-03"
      }
    }
  ]
}
```

Backend:

* Attaches `user_id` from token if present.
* Validates event types against allowed list.
* Inserts events into `telemetry_events` with batched insert.

### 7.2 Live Events & Seasons

#### `GET /v1/events/active`

* **Purpose:** Tell client which events/seasons apply right now.

**Response:**

```json
{
  "server_time": "2025-12-03T10:00:00Z",
  "events": [
    {
      "id": "event_fire_festival_01",
      "type": "event",
      "start_at": "2025-12-01T00:00:00Z",
      "end_at": "2025-12-07T00:00:00Z",
      "rules": { /* mutators, reward multipliers, etc. */ }
    },
    {
      "id": "season_shadow_architect_01",
      "type": "season",
      "start_at": "2025-11-01T00:00:00Z",
      "end_at": "2026-01-01T00:00:00Z",
      "rules": { /* season-level stuff */ }
    }
  ]
}
```

Client:

* Applies `events.rules` wherever relevant:

  * Adjusting draft weights (if rules say so).
  * Showing them on main menu/event banner.

### 7.3 Experiments

#### `GET /v1/experiments/assignments`

* **Auth:** required.
* **Purpose:** Tell client the variants this user is in.

**Response:**

```json
{
  "assignments": [
    {
      "experiment_id": "exp_draft_weights_v1",
      "variant_id": "more_relics",
      "parameters": {
        "draft_relic_weight_multiplier": 1.5
      }
    }
  ]
}
```

Client:

* Hooks `parameters` into its configuration layer:

  * Example: adjust weight when selecting draft choices.

Assignment logic:

* On first call:

  * Backend assigns deterministic variant using `user_id` hash and variant weights.
  * Stores in `experiment_assignments`.
* Future calls: returns stored assignment.

---

## 8. Live Balance Process

Concrete loop:

1. **Data collection**

   * Run daily/weekly scripts (Python) that:

     * Pull:

       * Run outcomes (from `runs`).
       * Telemetry (from `telemetry_events`).
       * Leaderboard stats.
2. **Analysis**

   * Scripts compute:

     * Per-content win contribution, pick rates.
     * Difficulty curves by tier and wave.
     * Event/experiment impact.
   * Flag:

     * Top 5 OP traps/monsters/relics.
     * 5 most underpowered ones.
     * Tiers with too high/low fail rates.
3. **Simulation**

   * Using Rust engine + content pack variants:

     * Try candidate buffs/nerfs.
     * Run Monte Carlo sims.
     * Compare distributions vs live data.
4. **Change proposal**

   * Output candidate patch:

     * Parameter diffs:

       * `trap_fire_glyph.base_damage 15 -> 12`
       * `monster_ember_guard.max_hp 100 -> 120` etc.
   * Produces a new authoring change (YAML/JSON).
5. **Patch build**

   * Run Phase 3 tools:

     * Compile new `content_pack.v1.1.0.json`.
   * Run engine tests & golden replays.
6. **Rollout**

   * Update:

     * `content/manifest` to new version.
   * Optionally:

     * Use experiments to soft-launch tuning changes to a subset before global rollout.
7. **Validation**

   * Compare pre/post patch metrics to ensure:

     * No huge regressions.
     * No new impossible spikes.

---

## 9. Release Management

Define version semantics:

* **Client:** `client_version = MAJOR.MINOR.PATCH`
* **Engine:** `engine_version` (matching crate/tag).
* **Content pack:** `content_version + content_hash`.

Rules:

* Backend `content_manifest` specifies:

  * `active_version`
  * `min_supported_client_version`
* Client on startup:

  * If `client_version < min_supported`, show “update required”.
  * If `content_version != active_version`, fetch new pack before allowing ranked/challenge submissions.

Rollback plan:

* If new engine/content causes issues:

  * Backend switches `active_version` back to previous.
  * Clients that already downloaded new pack:

    * Either:

      * Keep old packs cached and re-select appropriate one.
      * Or force a quick re-download of previous version.

Phase 6 target: **at least basic rollback** by flipping `active_version` and shipping previous pack.

---

## 10. Testing & QA

### 10.1 Backend

* Telemetry:

  * Tests for `POST /v1/telemetry/batch`:

    * Valid/invalid event types.
    * Batched inserts.
    * Auth/no-auth flows.
* Events:

  * Tests for `GET /v1/events/active` with overlapping events and edge time ranges.
* Experiments:

  * Tests for deterministic assignments:

    * Same user gets same variant across calls.
    * Distribution matches configured weights in aggregate.

### 10.2 Client

* Telemetry:

  * Ensure events are batched and sent without impacting gameplay.
  * Handle offline/failed telemetry silently, with retry/backoff.
* Events/Seasons:

  * UI surfacing of active events.
  * Correct application of event rules (mutators, bonuses).
* Experiments:

  * Variants map parameters correctly to client config.
  * Experiment flags do not crash the game when unknown parameter keys are added later.

### 10.3 Balance

* Golden sims:

  * Keep a curated suite of “balance tests” where:

    * You know expected win-rate bands and difficulty curves.
  * Run them automatically when content packs change.

---

## 11. Observability & SLOs

Define at least basic SLOs:

* **Backend:**

  * P99 latency < N ms for:

    * `/v1/run/submit`
    * `/v1/leaderboard`
  * Telemetry ingestion success rate > 99%.
* **Client:**

  * Crash rate per session < target threshold.
  * Telemetry overhead:

    * Event batch size and frequency not causing visible lag.

Dashboards (could be simple Grafana/whatever):

* Runs/day, validated vs invalid.
* Daily challenge participation.
* Leaderboard page views.
* Telemetry event volume trends.

---

## 12. Allowed Technical Debt in Phase 6

Allowed:

* Analytics on **cold storage in the main DB** for now, no big data infra.
* Simple experiments:

  * Only a few experiments running at once.
  * Single-layer parameters (no nested feature flags explosion).
* Live events:

  * Basic mutators and reward multipliers; no full “season pass” yet.

Not allowed:

* Shipping content/engine changes with no pathway to rollback.
* “Secret” live tuning that’s not represented in:

  * content packs,
  * experiments,
  * or event rules.
* Telemetry events with arbitrary unvalidated types (event schema must be controlled).

---

## 13. Definition of Done (Checklist)

Phase 6 is **done** when:

1. **Telemetry**

   * [ ] `telemetry_events` table created and populated from real clients.
   * [ ] `POST /v1/telemetry/batch` implemented, tested, and used by client.
   * [ ] Basic scripts exist to aggregate core KPIs (win rates, pick rates, challenge participation).

2. **Events & Seasons**

   * [ ] `live_events` table and a simple management flow (migration or admin tool).
   * [ ] `GET /v1/events/active` used by client to surface active events.
   * [ ] At least one test event/season successfully run internally.

3. **Experiments**

   * [ ] `experiments` and `experiment_assignments` tables implemented.
   * [ ] `GET /v1/experiments/assignments` integrated in client.
   * [ ] One small experiment (e.g., draft weight tweak) runs end-to-end and is measurable.

4. **Balance loop**

   * [ ] Python/Rust scripts pull telemetry & runs and generate balance reports.
   * [ ] Content patch pipeline from Phase 3 updated to support frequent tuning passes.
   * [ ] At least one balance patch shipped and its impact measured.

5. **Release management**

   * [ ] `content_manifest` used to control active content version.
   * [ ] Documented process to:

     * Ship new pack.
     * Roll back to previous pack.
   * [ ] Sane handling of version mismatches in client (no mysterious errors).

6. **Observability**

   * [ ] Core metrics and logs wired for runs, leaderboards, and telemetry ingestion.
   * [ ] Simple dashboards/set of queries ready for devs to use.

When all that’s ✅, Abyssal Architect isn’t just buildable – it’s **operable**: you can ship patches, run events, tune balance based on real data, and keep the game healthy long-term.
