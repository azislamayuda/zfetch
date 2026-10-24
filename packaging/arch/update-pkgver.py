#!/usr/bin/env python3
from pathlib import Path
import re
p = Path(__file__).resolve().parents[2]
cargo = (p / 'Cargo.toml').read_text()
match = re.search(r'^version\s*=\s*"(\d+\.\d+\.\d+)"', cargo, re.M)
if not match:
    raise SystemExit('Could not read version from Cargo.toml')
spec = p / 'packaging/arch/PKGBUILD'
text = spec.read_text()
text, count = re.subn(r'^pkgver=.*$', f'pkgver={match.group(1)}', text, count=1, flags=re.M)
if count != 1:
    raise SystemExit('Missing pkgver in PKGBUILD')
spec.write_text(text)
print('Arch pkgver:', match.group(1))
