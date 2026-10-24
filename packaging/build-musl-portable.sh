#!/usr/bin/env bash
# Package an already-built, natively testable static musl Rust binary.
set -euo pipefail
cd "$(dirname "$0")/.."
version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)"
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo 'Unexpected Cargo version' >&2; exit 1; }
arch="${PKG_ARCH:-$(uname -m)}"
target="${MUSL_TARGET:-${arch}-unknown-linux-musl}"
case "$arch:$target" in
  x86_64:x86_64-unknown-linux-musl|aarch64:aarch64-unknown-linux-musl) ;;
  *) echo "Invalid arch/target combination: $arch $target" >&2; exit 1 ;;
esac
[[ "$(uname -m)" == "$arch" ]] || { echo 'Smoke test needs a native runner for this architecture' >&2; exit 1; }
binary="target/$target/compact/zfetch"
[[ -x "$binary" ]] || { echo "Missing executable: $binary" >&2; exit 1; }
command -v readelf >/dev/null || { echo 'readelf is required' >&2; exit 1; }
command -v python3 >/dev/null || { echo 'python3 is required' >&2; exit 1; }
echo '=== MUSL BINARY ==='
file "$binary"
if readelf -l "$binary" | grep -q 'INTERP'; then
  echo 'FAIL: musl binary uses a dynamic interpreter' >&2
  exit 1
fi
if readelf -d "$binary" 2>/dev/null | grep -q '(NEEDED)'; then
  echo 'FAIL: musl binary requires shared libraries' >&2
  exit 1
fi
echo 'PASS: no ELF interpreter or dynamic shared-library requirement'
"$binary" --version | grep -Fx "zfetch $version"
"$binary" --no-logo --no-color >/dev/null
"$binary" --format json --privacy | python3 -m json.tool >/dev/null
staging="$(mktemp -d)"
trap 'rm -rf -- "$staging"' EXIT
folder="zfetch-$version-linux-$arch-musl"
mkdir -p "dist" "$staging/$folder"
install -m755 "$binary" "$staging/$folder/zfetch"
install -m644 README.md LICENSE "$staging/$folder/"
archive="dist/$folder.tar.gz"
tar -C "$staging" -czf "$archive" "$folder"
( cd dist && sha256sum "$folder.tar.gz" > "SHA256SUMS-$arch-musl" )
echo "Created: $archive"
