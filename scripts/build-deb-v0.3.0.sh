#!/usr/bin/env bash
# Build a local .deb + checksums without sudo, installation, Git tag, push or GitHub publication.
set -Eeuo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
expected_version='0.3.0'
expected_package='0.3.0-1'
if ! command -v cargo >/dev/null || ! command -v cargo-deb >/dev/null; then
  echo 'ERROR: cargo-deb missing. Install with: cargo install cargo-deb --locked' >&2
  exit 1
fi
for tool in dpkg-deb sha256sum tar; do
  command -v "$tool" >/dev/null || { echo "ERROR: Missing $tool" >&2; exit 1; }
done
version="$(python3 - <<'PY'
import pathlib,tomllib
print(tomllib.loads(pathlib.Path('Cargo.toml').read_text())['package']['version'])
PY
)"
if [[ "$version" != "$expected_version" ]]; then
  echo "ERROR: Cargo version $version != $expected_version" >&2
  exit 1
fi
if [[ ! -f docs/licenses/fastfetch-MIT.txt || ! -f docs/THIRD_PARTY_LOGOS.md ]]; then
  echo 'ERROR: third-party ASCII artwork attribution files missing' >&2
  exit 1
fi

echo '=== Package build ==='
cargo deb
DEB="target/debian/zfetch_${expected_package}_amd64.deb"
[[ -f "$DEB" ]] || { echo "ERROR: Expected package not found: $DEB" >&2; exit 1; }
[[ "$(dpkg-deb --field "$DEB" Package)" == 'zfetch' ]] || { echo 'ERROR: incorrect package name' >&2; exit 1; }
[[ "$(dpkg-deb --field "$DEB" Version)" == "$expected_package" ]] || { echo 'ERROR: incorrect package version' >&2; exit 1; }
[[ "$(dpkg-deb --field "$DEB" Architecture)" == 'amd64' ]] || { echo 'ERROR: expected amd64 package' >&2; exit 1; }

# Verify the actual packaged binary and attribution, not just the build directory.
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
dpkg-deb -x "$DEB" "$tmp"
[[ -x "$tmp/usr/bin/zfetch" ]] || { echo 'ERROR: packaged binary missing' >&2; exit 1; }
[[ "$("$tmp/usr/bin/zfetch" --version)" == 'zfetch 0.3.0' ]] || {
  echo 'ERROR: packaged executable has unexpected version' >&2; exit 1;
}
for name in LICENSE README.md fastfetch-MIT.txt THIRD_PARTY_LOGOS.md; do
  [[ -f "$tmp/usr/share/doc/zfetch/$name" ]] || {
    echo "ERROR: missing packaged documentation $name" >&2; exit 1;
  }
done

out="target/release-assets/v0.3.0"
mkdir -p "$out"
cp -- "$DEB" "$out/"
(
  cd "$out"
  sha256sum "$(basename "$DEB")" > SHA256SUMS
  sha256sum -c SHA256SUMS
)
echo '=== DEB fields ==='
dpkg-deb --field "$DEB" Package Version Architecture Depends
printf 'READY FOR MANUAL VALIDATION: %s\n' "$out"
echo 'This script did NOT install the .deb or publish a GitHub Release.'
echo 'Next: sudo apt-get -s install ./target/debian/zfetch_0.3.0-1_amd64.deb'
