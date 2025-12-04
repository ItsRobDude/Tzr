# Engine CLI and Simulation Guide

This crate exposes the `simulate_wave` API and a companion CLI for running
simulations from JSON dungeon/wave files.

## CLI usage

Builds ship a `sim_cli` binary with two subcommands:

- `run` executes a single wave: `cargo run --bin sim_cli -- run --dungeon path/to/dungeon.json --wave path/to/wave.json --seed 123`.
  - Flags: `--max-ticks` (default `MAX_TICKS`), `--summary-only` (hide events), and `--event-limit` (cap printed log entries).
- `stress` runs many seeds in one process: `cargo run --release --bin sim_cli -- stress --dungeon path/to/dungeon.json --wave path/to/wave.json --runs 500 --start-seed 10`.
  - Add `--verbose` to print per-seed results.

For shell convenience, `scripts/stress_wave.sh` wraps the stress command and
accepts optional `RUNS`, `START_SEED`, and `VERBOSE=1` environment variables.

## Data shapes

JSON inputs map directly onto the public structs exported by `engine`:

- `DungeonState` defines rooms (`RoomState`), edges, the core room, and starting
  core HP. Rooms contain `TrapInstance` and `UnitInstance` entries.
- `WaveConfig` lists `HeroSpawn` entries describing hero templates, counts,
  spawn rooms, and initial delays, plus any modifier tags.
- Units use the `UnitInstance` shape (faction, stats, HP, room, status effects,
  AI behavior, and attack cooldown). Traps follow `TrapInstance` (trigger type,
  cooldown, optional charges, damage, optional status-on-hit, tags).

Each struct includes JSON examples in its Rust doc comments if you need a
reference while authoring fixtures.

## Extending the simulator

New traps or hero templates should be added by extending the model definitions
and simulation logic together:

1. Add new fields or enum variants in `src/model` (for example `trap.rs` or
   `unit.rs`) to represent the data you need.
2. Update JSON fixtures or generation pipelines to emit the new shapes, keeping
   snake-case keys for compatibility.
3. Implement the behavior in the simulation loop under `src/sim`, adding events
   in `src/sim/events.rs` if you want them visible in CLI logs.
4. Update tests or create new fixtures in `src/sim/test_fixtures` to exercise
   the additions, then run `cargo test`.

The new CLI and stress script are good companions while iterating: use `run`
with a fixed seed to debug, and `stress` to ensure performance across a wide
range of RNG seeds.
