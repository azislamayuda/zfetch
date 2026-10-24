#!/usr/bin/env bash
# Fail if an ELF binary requires a GLIBC symbol newer than the given baseline.
# Usage: bash packaging/check-glibc.sh /path/to/binary 2.34
set -euo pipefail
if (( $# != 2 )); then
  echo 'Usage: check-glibc.sh BINARY MAX_GLIBC_VERSION' >&2
  exit 2
fi
binary="$1"
ceiling="$2"
[[ -f "$binary" ]] || { echo "Binary does not exist: $binary" >&2; exit 2; }
[[ "$ceiling" =~ ^[0-9]+\.[0-9]+$ ]] || { echo "Invalid version: $ceiling" >&2; exit 2; }
command -v readelf >/dev/null || { echo 'readelf is required' >&2; exit 2; }
# Dynamic binaries should expose GLIBC symbol version requirements.
versions="$(readelf --version-info "$binary" | grep -oE 'GLIBC_[0-9]+\.[0-9]+' || true)"
[[ -n "$versions" ]] || { echo 'No GLIBC symbols found: refusing unverified GNU baseline' >&2; exit 1; }
highest="$(printf '%s\n' "$versions" | sed 's/^GLIBC_//' | sort -Vu | tail -n 1)"
printf 'GLIBC required: %s, allowed maximum: %s\n' "$highest" "$ceiling"
if [[ "$(printf '%s\n' "$ceiling" "$highest" | sort -Vu | tail -n 1)" != "$ceiling" ]]; then
  echo "FAIL: binary requires GLIBC $highest, newer than $ceiling" >&2
  exit 1
fi
echo 'PASS: GLIBC baseline'
