#!/bin/bash
# Usage: stress-001.sh <test binary> <workers> <rounds> <filter> [--exact]
# Runs <workers> parallel loops of <rounds> runs of the filtered tests; prints one line per run and a total.
BIN=$1; WORKERS=$2; ROUNDS=$3; FILTER=$4; EXACT=$5
echo "binary: $BIN"
echo "sha256: $(shasum -a 256 "$BIN" | cut -d' ' -f1)"
echo "filter: $FILTER $EXACT; workers: $WORKERS; rounds per worker: $ROUNDS"
worker() {
  for r in $(seq 1 "$ROUNDS"); do
    out=$("$BIN" "$FILTER" $EXACT --test-threads=4 2>&1)
    code=$?
    summary=$(echo "$out" | grep "^test result" | head -1)
    echo "worker=$1 round=$r exit=$code $summary"
  done
}
for w in $(seq 1 "$WORKERS"); do worker "$w" & done
wait
