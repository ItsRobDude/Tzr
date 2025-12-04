#!/usr/bin/env bash
# Run many wave simulations across sequential seeds to stress-test performance.
# Usage: scripts/stress_wave.sh <dungeon.json> <wave.json> [runs] [start_seed]
set -euo pipefail

if [ "$#" -lt 2 ]; then
  echo "Usage: $0 <dungeon.json> <wave.json> [runs] [start_seed]" >&2
  exit 1
fi

DUNGEON="$1"
WAVE="$2"
RUNS="${3:-1000}"
START_SEED="${4:-1}"
MAX_TICKS="${MAX_TICKS:-60000}"
VERBOSE_FLAG=""
if [ "${VERBOSE:-}" != "" ]; then
  VERBOSE_FLAG="--verbose"
fi

cargo run --release --bin sim_cli -- \
  stress --dungeon "$DUNGEON" --wave "$WAVE" \
  --runs "$RUNS" --start-seed "$START_SEED" \
  --max-ticks "$MAX_TICKS" $VERBOSE_FLAG
