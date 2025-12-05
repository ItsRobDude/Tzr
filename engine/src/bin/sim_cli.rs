use std::env;
use std::fs;
use std::path::PathBuf;

use engine::model::SimulationOutcome;
use engine::sim::events::SimulationEvent;
use engine::sim::{MAX_EVENTS, MAX_TICKS};
use engine::{DungeonState, ENGINE_VERSION, WaveConfig, simulate_wave};

struct RunArgs {
    dungeon: PathBuf,
    wave: PathBuf,
    seed: u64,
    max_ticks: u32,
    summary_only: bool,
    event_limit: usize,
}

struct StressArgs {
    dungeon: PathBuf,
    wave: PathBuf,
    start_seed: u64,
    runs: u64,
    max_ticks: u32,
    verbose: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        return Ok(());
    };

    match command.as_str() {
        "run" => run_once(parse_run_args(args.collect())?)?,
        "stress" => stress(parse_stress_args(args.collect())?)?,
        _ => {
            eprintln!("unknown command: {command}\n");
            print_usage();
        }
    }

    Ok(())
}

fn parse_run_args(raw: Vec<String>) -> Result<RunArgs, String> {
    let mut dungeon = None;
    let mut wave = None;
    let mut seed = None;
    let mut max_ticks = MAX_TICKS;
    let mut summary_only = false;
    let mut event_limit = 0usize;

    let mut iter = raw.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--dungeon" => dungeon = Some(next_path(&arg, iter.next())?),
            "--wave" => wave = Some(next_path(&arg, iter.next())?),
            "--seed" => seed = Some(next_number(&arg, iter.next())?),
            "--max-ticks" => max_ticks = next_number(&arg, iter.next())?,
            "--summary-only" => summary_only = true,
            "--event-limit" => event_limit = next_number(&arg, iter.next())?,
            other => return Err(format!("unexpected argument: {other}")),
        }
    }

    Ok(RunArgs {
        dungeon: required("--dungeon", dungeon)?,
        wave: required("--wave", wave)?,
        seed: required("--seed", seed)?,
        max_ticks,
        summary_only,
        event_limit,
    })
}

fn parse_stress_args(raw: Vec<String>) -> Result<StressArgs, String> {
    let mut dungeon = None;
    let mut wave = None;
    let mut start_seed = 1u64;
    let mut runs = 100u64;
    let mut max_ticks = MAX_TICKS;
    let mut verbose = false;

    let mut iter = raw.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--dungeon" => dungeon = Some(next_path(&arg, iter.next())?),
            "--wave" => wave = Some(next_path(&arg, iter.next())?),
            "--start-seed" => start_seed = next_number(&arg, iter.next())?,
            "--runs" => runs = next_number(&arg, iter.next())?,
            "--max-ticks" => max_ticks = next_number(&arg, iter.next())?,
            "--verbose" => verbose = true,
            other => return Err(format!("unexpected argument: {other}")),
        }
    }

    Ok(StressArgs {
        dungeon: required("--dungeon", dungeon)?,
        wave: required("--wave", wave)?,
        start_seed,
        runs,
        max_ticks,
        verbose,
    })
}

fn required<T>(name: &str, value: Option<T>) -> Result<T, String> {
    value.ok_or_else(|| format!("missing required argument {name}"))
}

fn next_path(flag: &str, value: Option<String>) -> Result<PathBuf, String> {
    value
        .map(PathBuf::from)
        .ok_or_else(|| format!("expected value after {flag}"))
}

fn next_number<T>(flag: &str, value: Option<String>) -> Result<T, String>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    value
        .ok_or_else(|| format!("expected value after {flag}"))?
        .parse()
        .map_err(|err| format!("failed to parse {flag}: {err}"))
}

fn run_once(args: RunArgs) -> Result<(), Box<dyn std::error::Error>> {
    let dungeon: DungeonState = load_json(&args.dungeon)?;
    let wave: WaveConfig = load_json(&args.wave)?;

    let result = simulate_wave(dungeon, wave, args.seed, args.max_ticks)?;

    print_summary(
        args.seed,
        &result.outcome,
        &result.final_dungeon.core_hp,
        &result.stats,
        result.events.len(),
    );

    if !args.summary_only {
        let total_events = result.events.len();
        let limit = if args.event_limit == 0 {
            total_events
        } else {
            args.event_limit.min(total_events)
        };

        println!("\nEvent log (showing {limit} of {total_events}):");
        for event in result.events.iter().take(limit) {
            println!("- {}", format_event(event));
        }
        if limit < total_events {
            println!("... {} more events omitted ...", total_events - limit);
        }
    }

    Ok(())
}

fn stress(args: StressArgs) -> Result<(), Box<dyn std::error::Error>> {
    let dungeon: DungeonState = load_json(&args.dungeon)?;
    let wave: WaveConfig = load_json(&args.wave)?;

    let mut heroes_won = 0u64;
    let mut dungeon_won = 0u64;
    let mut timed_out = 0u64;

    for offset in 0..args.runs {
        let seed = args.start_seed + offset;
        let result = simulate_wave(dungeon.clone(), wave.clone(), seed, args.max_ticks)?;
        match result.outcome {
            SimulationOutcome::DungeonWin => dungeon_won += 1,
            SimulationOutcome::HeroesWin => heroes_won += 1,
            SimulationOutcome::Timeout => timed_out += 1,
        }

        if args.verbose {
            println!(
                "seed {seed}: {:?} (ticks: {}, hero kills: {}, monster kills: {})",
                result.outcome,
                result.stats.ticks_run,
                result.stats.heroes_killed,
                result.stats.monsters_killed
            );
        }
    }

    println!(
        "Ran {} simulations starting at seed {}",
        args.runs, args.start_seed
    );
    println!("Dungeon wins: {dungeon_won}");
    println!("Hero wins: {heroes_won}");
    println!("Timeouts: {timed_out}");

    Ok(())
}

fn load_json<T: serde::de::DeserializeOwned>(
    path: &PathBuf,
) -> Result<T, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(path)?;
    let parsed = serde_json::from_str(&data)?;
    Ok(parsed)
}

fn print_summary(
    seed: u64,
    outcome: &SimulationOutcome,
    core_hp: &i32,
    stats: &engine::model::SimulationStats,
    event_count: usize,
) {
    println!("Engine version: {ENGINE_VERSION}");
    println!("Seed: {seed}");
    println!("Outcome: {:?}", outcome);
    println!("Core HP after wave: {core_hp}");
    println!("Ticks run: {}", stats.ticks_run);
    println!("Heroes spawned: {}", stats.heroes_spawned);
    println!("Heroes killed: {}", stats.heroes_killed);
    println!("Monsters killed: {}", stats.monsters_killed);
    println!("Total damage to core: {}", stats.total_damage_to_core);
    println!("Events emitted: {event_count} / {MAX_EVENTS}");
}

fn format_event(event: &SimulationEvent) -> String {
    match event {
        SimulationEvent::UnitSpawned {
            tick,
            unit_id,
            room_id,
        } => {
            format!(
                "[t={tick}] Unit {:?} spawned in room {:?}",
                unit_id, room_id
            )
        }
        SimulationEvent::UnitMoved {
            tick,
            unit_id,
            from,
            to,
        } => {
            format!(
                "[t={tick}] Unit {:?} moved from room {:?} to {:?}",
                unit_id, from, to
            )
        }
        SimulationEvent::TrapTriggered {
            tick,
            trap_id,
            room_id,
        } => {
            format!(
                "[t={tick}] Trap {:?} triggered in room {:?}",
                trap_id, room_id
            )
        }
        SimulationEvent::DamageApplied {
            tick,
            source,
            target,
            amount,
        } => match source {
            Some(src) => format!(
                "[t={tick}] Unit {:?} dealt {amount} damage to {:?}",
                src, target
            ),
            None => format!("[t={tick}] {amount} damage applied to {:?}", target),
        },
        SimulationEvent::StatusApplied { tick, target, kind } => {
            format!("[t={tick}] Status {:?} applied to {:?}", kind, target)
        }
        SimulationEvent::UnitDied { tick, unit_id } => {
            format!("[t={tick}] Unit {:?} died", unit_id)
        }
        SimulationEvent::CoreDamaged {
            tick,
            amount,
            core_hp_after,
        } => format!("[t={tick}] Core took {amount} damage (hp now {core_hp_after})"),
    }
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!(
        "  sim_cli run --dungeon <file> --wave <file> --seed <n> [--max-ticks <n>] [--summary-only] [--event-limit <n>]"
    );
    eprintln!(
        "  sim_cli stress --dungeon <file> --wave <file> [--start-seed <n>] [--runs <n>] [--max-ticks <n>] [--verbose]"
    );
}
