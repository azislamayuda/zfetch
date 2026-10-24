#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
command -v rpmbuild >/dev/null || { echo 'Install rpm tools first' >&2; exit 1; }
version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1)"
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo 'Unexpected Cargo version' >&2; exit 1; }
arch="${PKG_ARCH:-$(uname -m)}"
rpm_arch="${RPM_ARCH:-$arch}"
case "$arch" in x86_64|aarch64) ;; *) echo "Unsupported arch: $arch" >&2; exit 1;; esac
binary="$PWD/target/compact/zfetch"
[[ -x "$binary" ]] || { echo "Missing binary: $binary" >&2; exit 1; }
mkdir -p dist
staging="$(mktemp -d)"
trap 'rm -rf "$staging"' EXIT
mkdir -p "$staging/root/usr/bin" "$staging/root/usr/share/doc/zfetch" "$staging/root/usr/share/licenses/zfetch" "$staging/rpmbuild"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
install -m755 "$binary" "$staging/root/usr/bin/zfetch"
install -m644 README.md "$staging/root/usr/share/doc/zfetch/README.md"
install -m644 LICENSE "$staging/root/usr/share/licenses/zfetch/LICENSE"
cat > "$staging/rpmbuild/SPECS/zfetch.spec" <<SPEC
Name: zfetch
Version: $version
Release: 1%{?dist}
Summary: Lightweight Linux system information CLI written in Rust
License: MIT
URL: https://github.com/azislamayuda/zfetch
BuildArch: $rpm_arch
AutoReqProv: yes
%description
Fast native system information tool with configurable ASCII logos, JSON, and privacy features.
%install
mkdir -p %{buildroot}/usr/bin %{buildroot}/usr/share/doc/zfetch %{buildroot}/usr/share/licenses/zfetch
install -m755 $staging/root/usr/bin/zfetch %{buildroot}/usr/bin/zfetch
install -m644 $staging/root/usr/share/doc/zfetch/README.md %{buildroot}/usr/share/doc/zfetch/README.md
install -m644 $staging/root/usr/share/licenses/zfetch/LICENSE %{buildroot}/usr/share/licenses/zfetch/LICENSE
%files
/usr/bin/zfetch
%doc /usr/share/doc/zfetch/README.md
%license /usr/share/licenses/zfetch/LICENSE
SPEC
rpmbuild -bb --target "$rpm_arch" --define "_topdir $staging/rpmbuild" "$staging/rpmbuild/SPECS/zfetch.spec"
find "$staging/rpmbuild/RPMS" -name '*.rpm' -exec cp -v '{}' dist/ \;
# Portable GNU binary tarball; still requires compatible glibc/runtime libraries.
mkdir -p "$staging/portable/zfetch-$version-linux-$arch"
install -m755 "$binary" "$staging/portable/zfetch-$version-linux-$arch/zfetch"
install -m644 README.md LICENSE "$staging/portable/zfetch-$version-linux-$arch/"
tar -C "$staging/portable" -czf "dist/zfetch-$version-linux-$arch-gnu.tar.gz" "zfetch-$version-linux-$arch"
( cd dist && sha256sum -- *.rpm *.tar.gz > "SHA256SUMS-$arch" )
rpm -qp --queryformat '%{NAME} %{VERSION} %{ARCH}\n' dist/*.rpm
file "$binary"
