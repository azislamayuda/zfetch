#!/usr/bin/env python3
"""Offline smoke/regression checks for the zfetch v0.3.0 CLI.

Usage: python3 scripts/smoke-v0.3.0.py [path/to/zfetch]
Network-dependent --check-update/--update are deliberately excluded.
"""
from __future__ import annotations
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import tomllib


class SmokeFailure(Exception):
    pass


def check(ok: bool, message: str) -> None:
    if not ok:
        raise SmokeFailure(message)


def main() -> int:
    path = Path(sys.argv[1] if len(sys.argv) > 1 else 'target/compact/zfetch').expanduser().resolve()
    if not path.is_file() or not os.access(path, os.X_OK):
        print(f'ERROR: executable not found: {path}', file=sys.stderr)
        return 2
    with tempfile.TemporaryDirectory(prefix='zfetch-smoke-') as directory:
        config_home = Path(directory)
        # Isolated XDG_CONFIG_HOME preserves the user's MJ logo and configuration.
        environment = os.environ.copy()
        environment['XDG_CONFIG_HOME'] = str(config_home)
        environment['NO_COLOR'] = '1'
        environment['LC_ALL'] = 'C.UTF-8'

        def run(*args: str, expected: int = 0) -> subprocess.CompletedProcess[str]:
            result = subprocess.run([str(path), *args], env=environment, text=True,
                                    capture_output=True, timeout=25, check=False)
            if result.returncode != expected:
                raise SmokeFailure(f'zfetch {" ".join(args)} -> {result.returncode}, expected {expected}\n'
                                   f'stdout: {result.stdout[-1000:]}\nstderr: {result.stderr[-1000:]}')
            return result

        expected_version = tomllib.loads(
            (Path(__file__).resolve().parents[1] / 'Cargo.toml')
            .read_text(encoding='utf-8')
        )['package']['version']
        version = run('--version').stdout.strip()
        check(version == f'zfetch {expected_version}', f'unexpected version: {version}')
        print(f'[PASS] Version v{expected_version}')

        help_text = run('--help').stdout
        check(all(flag in help_text for flag in ('--format', '--only', '--layout', '--hardware-details')),
              'expected Sprint 01-03 flags missing from --help')
        print('[PASS] Help includes new options')

        modules = run('--list-modules').stdout
        check('cpu' in modules and 'kernel' in modules, '--list-modules missing expected names')
        themes = run('--list-themes').stdout
        check('nord' in themes and 'rust' in themes, '--list-themes missing themes')
        print('[PASS] Modules and themes')

        compact = run('--layout', 'compact', '--preset', 'minimal').stdout
        check(f'zfetch v{expected_version}' in compact and 'SYSTEM INFO' not in compact and 'HARDWARE INFO' not in compact,
              'compact layout contains decorative headings or no version')
        detailed = run('--layout', 'detailed', '--only', 'cpu,gpu,display,battery').stdout
        check(f'zfetch v{expected_version}' in detailed and 'HARDWARE INFO' in detailed,
              'detailed layout failed')
        print('[PASS] Compact/detailed layouts')

        raw = run('--format', 'json', '--only', 'os,kernel,cpu,memory').stdout
        check('\x1b' not in raw, 'ANSI escape leaked into JSON')
        document = json.loads(raw)
        check(document.get('schema_version') == 1, 'JSON schema_version changed')
        check(document.get('zfetch_version') == expected_version, 'JSON version mismatch')
        check(set(document['system']) == {'os', 'kernel'}, f'unexpected system JSON keys: {document["system"]}')
        check(set(document['hardware']) == {'cpu', 'memory'}, f'unexpected hardware JSON keys: {document["hardware"]}')
        print('[PASS] JSON schema and --only output')

        extended = json.loads(run('--format', 'json', '--only', 'cpu,gpu,memory', '--hardware-details').stdout)
        check('cpu' in extended['hardware'] and 'gpu' in extended['hardware'],
              'JSON hardware details output unavailable')
        print('[PASS] JSON hardware output')

        wrong = run('--layout', 'detailed', '--format', 'json', expected=2)
        check('terminal-only' in wrong.stderr or 'JSON' in wrong.stderr,
              'invalid JSON + detailed layout lacks explanation')
        error = run('--only', 'not_a_module', expected=2)
        check('module' in error.stderr.lower(), 'unknown module is not rejected')
        print('[PASS] Invalid option combinations are rejected')

        config = config_home / 'zfetch' / 'config.jsonc'
        run('--init-config')
        check(config.is_file(), '--init-config did not create config.jsonc')
        run('--validate-config')
        double_init = run('--init-config', expected=2)
        check('exist' in double_init.stderr.lower() or 'create' in double_init.stderr.lower(),
              'second --init-config did not safely refuse overwrite')
        print('[PASS] Config create/validate/no-overwrite')

        logo = config_home / 'smoke-logo.txt'
        logo.write_text('ZFETCH-SMOKE-LOGO\n', encoding='utf-8')
        displayed = run('--logo', str(logo), '--only', 'os').stdout
        check('ZFETCH-SMOKE-LOGO' in displayed, 'custom ASCII logo missing')
        hidden = run('--logo', str(logo), '--no-logo', expected=2)
        check('logo' in hidden.stderr.lower(), 'conflicting logo flags not rejected')
        print('[PASS] Custom logo and conflicting flags')
        print('PASS: all offline Sprint 04 smoke checks passed.')
    return 0


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (SmokeFailure, OSError, json.JSONDecodeError, subprocess.TimeoutExpired) as exc:
        print(f'FAIL: {exc}', file=sys.stderr)
        sys.exit(1)
