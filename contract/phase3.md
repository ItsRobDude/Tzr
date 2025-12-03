# Abyssal Architect – Phase 3

Content, Data & Tooling – Design Contract

---

## 1. Purpose & Scope

Phase 3 turns Abyssal Architect from a code-driven prototype into a **content-driven game**.

It is responsible for:

* Defining **data schemas** for:

  * Traps, monsters, heroes, rooms, waves, relics/modifiers, and tags.
* Creating a **content pipeline**:

  * Author content in human-editable formats (YAML/JSON/CSV).
  * Validate and compile into a single `content_pack.json` (or similar) for:

    * The Rust engine (Phase 1).
    * The TypeScript client (Phase 2).
* Providing **Python tooling**:

  * Validation (schema + semantic).
  * Sanity checks (no impossible combos, broken references).
  * Batch simulations for basic balancing using the native engine.
* Wiring content into the client:

  * Phase 2’s draft system and UI stop using hardcoded stub data and consume real content.

Phase 3 is **not** responsible for:

* Backend storage or user accounts.
* Meta-progression logic (what unlocks when).
* Live telemetry from real players (that’s Phase 6).
* Fancy UX for content editing (no in-game editor yet; tooling is dev-facing).

---

## 2. Dependencies & Boundaries

### 2.1 Depends on

* **Phase 1 – Engine**

  * Has stable types for:

    * `DungeonState`, `WaveConfig`, unit configs, trap configs, status effects.
  * Accepts configuration values (HP, damage, tags, etc.) that we can feed from data files.
* **Phase 2 – Client**

  * Has:

    * Draft system and run-state machinery.
    * Dungeon rendering and wave playback.
  * Is currently wired to “dummy” data or minimal content.

### 2.2 Provides to

* **Phase 2 (Client)**:

  * A `content_pack.json` that describes:

    * Available traps/monsters/heroes and their stats.
    * Which drafts are possible at each tier.
    * Default wave definitions (unless waves are partially procedural).
* **Phase 4 (Backend)**:

  * A versioned content pack that backend can serve to clients.
  * Shared IDs for content so backend can understand runs.

All systems must treat the **content pack as the single source of truth** for game content.

---

## 3. Tech Stack & Project Layout

### 3.1 Languages & Tools

* **Python** (3.10+):

  * Content compiler & validator.
  * Balancing scripts (calling native engine).
* **Rust**:

  * Engine’s config structs annotated for serialization (`serde`).
  * Optional CLI utilities for sim/balance that Python can call.
* **TypeScript**:

  * Types mirroring the content schema (generated or hand-written).
  * Loader for `content_pack.json`.

### 3.2 Suggested Structure

At the repo level (conceptual):

```text
/engine/          # Rust engine (Phase 1)
/client/          # TS client (Phase 2)
/content/         # New for Phase 3
  authoring/
    traps.yaml
    monsters.yaml
    heroes.yaml
    rooms.yaml
    waves.yaml
    relics.yaml
    modifiers.yaml
    tags.yaml
  compiled/
    content_pack.v1.json
  schema/
    content_schema.json
  tools/
    compile_content.py
    validate_content.py
    balance_sims.py
    report_balance.py
```

---

## 4. Content Model & Schemas

### 4.1 Global Constraints

* Every content entity has:

  * A **stable ID** (string): used across engine, client, backend.
  * A human-readable `name`.
  * Optional `tags: string[]` for synergy and filtering.
* All IDs must be unique across their category.
* Versioning:

  * `content_pack` has a `version` and `content_hash`.
  * Engine & client include the loaded content version in debug info.

### 4.2 Traps

Minimal trap schema:

```yaml
# content/authoring/traps.yaml
- id: "trap_fire_glyph"
  name: "Fire Glyph"
  description: "Triggers when heroes enter, scorching them with fire damage."
  rarity: "common"        # common | uncommon | rare | epic (for draft weighting)
  base_damage: 15
  damage_type: "fire"     # matches engine enum/tag
  cooldown_ticks: 10
  trigger_type: "on_enter" # on_enter | on_exit | timed | manual (if ever)
  max_charges: 999        # or null for infinite
  tags: ["fire", "aoe", "burst"]
  target_pattern: "room_all_heroes" # maps to engine targeting patterns
  unlock_tier: 0          # meta-progression later; for now 0 means always available
```

Python tooling will:

* Validate numeric ranges.
* Ensure `damage_type`, `trigger_type`, `target_pattern` are valid engine values.
* Enforce uniqueness of IDs.

### 4.3 Monsters

```yaml
# content/authoring/monsters.yaml
- id: "monster_ember_guard"
  name: "Ember Guard"
  description: "Slow but tanky defender that burns attackers."
  rarity: "common"
  max_hp: 100
  armor: 5
  move_speed: 0.5           # tiles per second equivalent; mapped to ticks
  attack_damage: 10
  attack_interval_ticks: 6
  attack_range: 1           # in rooms or tiles, per engine model
  ai_behavior: "guard_room" # guard_room | patrol | chase | ranged_support
  status_on_hit:
    type: "burn"
    duration_ticks: 12
    intensity: 3
  tags: ["fire", "tank", "frontline"]
  unlock_tier: 0
```

### 4.4 Heroes

```yaml
# content/authoring/heroes.yaml
- id: "hero_knight"
  name: "Knight"
  role: "frontline"           # frontline | assassin | support | caster
  max_hp: 120
  armor: 8
  move_speed: 1.0
  attack_damage: 8
  attack_interval_ticks: 5
  attack_range: 1
  abilities:
    - id: "hero_knight_shield_block"
      trigger: "on_hit"
      effect: "reduce_incoming_damage"
      magnitude: 0.5         # 50%
      cooldown_ticks: 10
  tags: ["human", "physical"]
```

### 4.5 Rooms

```yaml
# content/authoring/rooms.yaml
- id: "room_corridor_basic"
  name: "Stone Corridor"
  base_capacity_monsters: 2
  base_capacity_traps: 2
  tags: ["neutral", "corridor"]
  special_rules: []          # reserved for later (e.g., innate modifiers)
```

Dungeon generation (how these are arranged) can be separate; Phase 3 just defines room *types*.

### 4.6 Waves

Waves define how heroes come in. Example:

```yaml
# content/authoring/waves.yaml
- id: "wave_tier1_intro"
  tier: 1
  entries:
    - hero_id: "hero_knight"
      count: 4
      spawn_room_id: "entrance"
      delay_ticks: 0
    - hero_id: "hero_archer"
      count: 2
      spawn_room_id: "entrance"
      delay_ticks: 10
  modifiers: ["wave_modifier_no_healers"] # optional global tweaks
  tags: ["intro", "physical_focus"]
```

### 4.7 Relics / Global Modifiers

These are draftable “artifacts”:

```yaml
# content/authoring/relics.yaml
- id: "relic_furnace_core"
  name: "Furnace Core"
  description: "All fire damage is increased by 20%."
  effect_type: "global_multiplier"
  effect_target_tag: "fire"
  effect_multiplier: 1.2
  tags: ["fire", "scaling"]
  unlock_tier: 0
```

### 4.8 Tags

Optional separate list to keep tags consistent:

```yaml
# content/authoring/tags.yaml
- id: "fire"
  description: "Fire-based damage and effects."
- id: "poison"
  description: "Damage-over-time based on toxins."
- id: "control"
  description: "Stuns, slows, and positional effects."
```

---

## 5. Content Pipeline & Tools

### 5.1 compile_content.py

**Goal:** Turn all `authoring/*.yaml` into a single `compiled/content_pack.v1.json`.

Responsibilities:

* Load all YAML files.
* Validate against schema (see below).
* Resolve references:

  * Ensure all referenced hero IDs, room IDs, tags, etc. exist.
* Normalize:

  * Fill defaults.
  * Compute any derived fields (e.g., DPS estimates, draft weights).
* Produce:

```jsonc
{
  "version": "1.0.0",
  "content_hash": "sha256:...",
  "traps": [...],
  "monsters": [...],
  "heroes": [...],
  "rooms": [...],
  "waves": [...],
  "relics": [...],
  "modifiers": [...],
  "tags": [...]
}
```

Constraints:

* Must be deterministic:

  * Same authoring input → same `content_pack` and `content_hash`.
* Fail-fast on:

  * Missing IDs.
  * Duplicate IDs.
  * Invalid enum values.
  * Bad numeric ranges (HP ≤ 0, negative cooldown, etc.).

### 5.2 validate_content.py

**Goal:** Run **schema + semantic checks** independent of compilation.

Checks:

* Schema validation:

  * Use JSON schema (in `/content/schema`) or pydantic models.
* ID references:

  * `trap_x` that references damage type `fire` must match known damage types engine supports.
  * Waves referencing hero IDs that exist.
* Tag usage:

  * Warn if tags are defined but never used.
  * Warn if content uses tags not defined in `tags.yaml`.
* Rarity distribution:

  * Warn if too many items in high rarity causing poor draft diversity.
* Draft viability:

  * Ensure each tier has enough draftable items to avoid dead pools.

Outputs:

* Exit code non-zero on **hard errors**.
* Human-readable report of **warnings** and **errors**.

### 5.3 balance_sims.py

**Goal:** Fire-and-forget balancer that runs thousands of simulated waves/runs using the native Rust engine and summarized content.

Responsibilities:

* Consume `content_pack.v1.json`.
* Generate test scenarios:

  * For each tier:

    * Sample N random dungeon builds (within rules).
    * Simulate M random waves per build.
  * Or simulate short runs with random draft choices.
* Call native engine:

  * Via CLI: `engine_sim --input scenario.json --seed 1234`.
  * Or via Python FFI, whichever is easier/safer.
* Collect metrics:

  * Hero survival rates per wave.
  * Average damage dealt by traps/monsters (DPS).
  * Win rates at each tier.
  * Pick-rate proxies: how often certain traps/monsters appear in winning builds.

### 5.4 report_balance.py

**Goal:** Turn raw sim data into actionable summaries.

Outputs:

* Per trap/monster:

  * Avg usage in winning runs.
  * Impact metrics (avg damage done, etc).
  * Rating (OP, underpowered, okay) based on thresholds.
* Per hero:

  * Survivability across tiers.
  * Which builds they consistently crush/struggle against.
* Per tier:

  * Win-rate distribution, difficulty spikes.

Reports:

* Machine-readable (JSON/CSV).
* Human-readable (Markdown/HTML summary) for design review.

---

## 6. Integration with Engine & Client

### 6.1 Engine (Rust)

Requirements:

* Engine must have **config structs** that correspond to content schema types:

```rust
struct TrapConfig { /* ... */ }
struct MonsterConfig { /* ... */ }
struct HeroConfig { /* ... */ }
// ...
struct ContentPack {
    traps: Vec<TrapConfig>,
    monsters: Vec<MonsterConfig>,
    heroes: Vec<HeroConfig>,
    // ...
}
```

* Load content pack:

  * From JSON (for CLI tools) or from embedded data (for WASM).
* The engine must be able to:

  * Map IDs to configs quickly (e.g., hash map).
  * Use content configs in simulation rather than hard-coded values.

### 6.2 Client (TS)

Requirements:

* Define TS types mirroring the content schema:

```ts
interface TrapDef { /* ... */ }
interface MonsterDef { /* ... */ }
interface HeroDef { /* ... */ }
interface ContentPack { /* ... */ }
```

* Load `content_pack.v1.json` at startup:

  * Either bundled at build time or fetched locally.
* Use content pack to:

  * Build draft pools.
  * Show names/descriptions in UI tooltips.
  * Display tags and synergy hints.
  * Feed numeric stats into any client-side previews (e.g., “approx damage”).

**Important:** Client does **not** compute actual combat; it just displays content and sends the right configs/IDs into the engine.

---

## 7. Testing & QA

### 7.1 Unit Tests (Python)

* For compiler:

  * Valid content compiles → expected `content_pack`.
  * Invalid content (e.g. bad ID, negative HP) → failure.
* For validator:

  * Known bad YAML fixtures produce the expected errors.
  * Edge case: empty or minimal content pack yields useful error, not crash.

### 7.2 Integration Tests

* End-to-end pipeline:

  * Given known authoring files:

    * `compile_content.py` produces an expected `content_pack`.
    * Client loads that pack and can start a run using it.
* Engine integration:

  * Load compiled content into engine via CLI and ensure:

    * No panics on startup.
    * Traps/monsters/heroes from the pack can be instantiated.

### 7.3 Balancing Sanity

Even with minimal content, run a few hundred sims to ensure:

* No “instant loss” waves at early tiers for average builds.
* No hero/trap/monster with absurd outlier stats (e.g., >10x the rest).

---

## 8. Observability & Debugging

Tooling must:

* Print clear, human-readable errors:

  * Include file path and line/entry that failed.
* Provide a **“lint-only”** mode:

  * Show warnings about design issues (e.g., too many rare items).
* Provide a **“diff mode”**:

  * Compare two versions of a `content_pack`:

    * Show added/removed/changed traps/monsters/heroes.
    * Useful when balancing.

Client dev features:

* Debug menu entry:

  * Show current loaded `content_pack.version` and `content_hash`.
  * List a small summary (counts of traps/monsters/heroes/waves).

---

## 9. Allowed Technical Debt in Phase 3

Allowed:

* Simpler wave definitions initially:

  * Hardcoded sequences instead of full procedural waves; we can layer procedural generation later.
* Basic balance_sims:

  * Monte Carlo “spray and pray” over builds instead of optimal/ML-driven search.
* Minimal markdown/HTML report formatting:

  * As long as the data is useful and readable.

Not allowed:

* Hard-coded content in engine or client once the content pipeline exists.
* Silent failures in compiler/validator (must fail with actionable messages).
* Changing content IDs casually – once live, IDs are part of save/replay/telemetry; treat them as persistent.

---

## 10. Definition of Done (Checklist)

Phase 3 is **done** when:

1. **Schemas & Authoring**

   * [ ] YAML/JSON schemas exist for traps, monsters, heroes, rooms, waves, relics, modifiers, tags.
   * [ ] At least a **minimal content set** is authored using those schemas.

2. **Compiler & Validator**

   * [ ] `compile_content.py` produces a single `content_pack.v1.json` from authoring files.
   * [ ] `validate_content.py` catches malformed content with clear errors.
   * [ ] Compiler and validator are wired into CI to prevent broken content merges.

3. **Engine Integration**

   * [ ] Rust engine can load `content_pack.v1.json` and instantiate configs.
   * [ ] Hard-coded stats in engine are removed or guarded (content-driven for implemented features).

4. **Client Integration**

   * [ ] Client loads content pack at startup.
   * [ ] Draft choices are drawn from content pack (rarities/tiers respected).
   * [ ] UI shows names/descriptions/tags from content pack, not hard-coded strings.

5. **Balancing Tools**

   * [ ] `balance_sims.py` can run batches of sims using the content pack and native engine.
   * [ ] `report_balance.py` outputs a basic but useful summary of balance metrics.

6. **Docs**

   * [ ] “How to add a new trap/monster/hero” doc for designers.
   * [ ] README in `/content/` explaining authoring → compile → client/engine integration.

Once all of this is ✅, content design becomes a data problem, not a coding problem, and you’re ready for Phase 4 (Backend & Meta-Progression) to hook into a clean, versioned content world.

