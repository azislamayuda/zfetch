#!/usr/bin/env bash
# Controlled comparison; measurements vary with system load.
set -Eeuo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
BIN="${1:-./target/compact/zfetch}"
BASELINE="${2:-}"
if [[ ! -x "$BIN" ]]; then
  echo "ERROR: binary not executable: $BIN" >&2
  exit 1
fi
if ! command -v hyperfine >/dev/null; then
  echo 'ERROR: hyperfine missing (sudo apt install hyperfine)' >&2
  exit 1
fi
if [[ ! -x /usr/bin/time ]]; then
  echo 'ERROR: /usr/bin/time missing (sudo apt install time)' >&2
  exit 1
fi
if [[ -n "$BASELINE" && ! -x "$BASELINE" ]]; then
  echo "ERROR: baseline binary not executable: $BASELINE" >&2
  exit 1
fi
out="target/benchmarks/sprint04-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$out"
BIN="$(realpath "$BIN")"
if [[ -n "$BASELINE" ]]; then BASELINE="$(realpath "$BASELINE")"; fi
{
  printf 'Date: %s\n' "$(date -Is)"
  printf 'OS: %s\n' "$(uname -a)"
  printf 'Current binary: %s (%s bytes)\n' "$BIN" "$(stat -c '%s' "$BIN")"
  "$BIN" --version
  if [[ -n "$BASELINE" ]]; then
    printf 'Baseline binary: %s (%s bytes)\n' "$BASELINE" "$(stat -c '%s' "$BASELINE")"
    "$BASELINE" --version
  fi
} | tee "$out/metadata.txt"

# Same flags for a like-for-like default comparison. Does not enable v0.3-only features.
if [[ -n "$BASELINE" ]]; then
  echo '=== Version A/B, same default module selection ==='
  hyperfine --shell=none --warmup 20 --runs 100 \
    --export-json "$out/default-comparison.json" \
    "$BASELINE --no-logo --no-color" \
    "$BIN --no-logo --no-color"
else
  echo '=== v0.3.0 default startup ==='
  hyperfine --shell=none --warmup 20 --runs 100 \
    --export-json "$out/default.json" \
    "$BIN --no-logo --no-color"
fi

# This intentionally compares mode overhead, not equivalent module sets.
echo '=== Optional hardware-details cost ==='
hyperfine --shell=none --warmup 20 --runs 100 \
  --export-json "$out/hardware-cost.json" \
  "$BIN --no-logo --no-color" \
  "$BIN --no-logo --no-color --hardware-details"

echo '=== Peak resident memory, 10 independent executions (KiB) ==='
bins=("$BIN")
if [[ -n "$BASELINE" ]]; then bins=("$BASELINE" "$BIN"); fi
for bin in "${bins[@]}"; do
  label="$(basename "$bin")-$(printf '%s' "$bin" | sha256sum | cut -c1-8)"
  : > "$out/${label}-rss-kib.txt"
  for _ in $(seq 1 10); do
    /usr/bin/time -f '%M' -o "$out/peak.txt" "$bin" --no-logo --no-color >/dev/null
    cat "$out/peak.txt" >> "$out/${label}-rss-kib.txt"
  done
  awk -v label="$label" '{total+=$1} END {printf "%s average RSS: %.1f KiB over %d runs\n", label, total/NR, NR}' \
    "$out/${label}-rss-kib.txt" | tee -a "$out/metadata.txt"
done
rm -f "$out/peak.txt"
echo "Results: $out"
echo 'Reminder: do not compare runs with differing enabled modules as an equivalent-feature benchmark.'
