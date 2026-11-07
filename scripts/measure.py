#!/usr/bin/env python3
"""Peak RSS and median wall time of zfetch over N paired runs.

Usage: python3 scripts/measure.py [binary[,binary2]] [runs] [-- extra args]
Default args: --no-logo --no-color. Two comma-separated binaries are run interleaved (A/B).
RSS comes from /usr/bin/time -f %M: ru_maxrss read via Python's wait4 includes the
forked Python interpreter and reports the same inflated value for every binary.
"""
import os
import statistics
import subprocess
import sys
import time


def wall_ms(binary: str, args: list[str]) -> float:
    null = os.open(os.devnull, os.O_WRONLY)
    actions = [(os.POSIX_SPAWN_DUP2, null, 1), (os.POSIX_SPAWN_DUP2, null, 2)]
    start = time.perf_counter()
    pid = os.posix_spawn(binary, [binary, *args], os.environ, file_actions=actions)
    _, status, _ = os.wait4(pid, 0)
    elapsed = time.perf_counter() - start
    os.close(null)
    if os.waitstatus_to_exitcode(status) != 0:
        raise SystemExit(f'{binary} exited with {status}')
    return elapsed * 1000


def rss_kib(binary: str, args: list[str]) -> int:
    result = subprocess.run(['/usr/bin/time', '-f', '%M', binary, *args],
                            stdout=subprocess.DEVNULL, stderr=subprocess.PIPE,
                            text=True, check=True)
    return int(result.stderr.strip().splitlines()[-1])


def main() -> int:
    argv = sys.argv[1:]
    extra = ['--no-logo', '--no-color']
    if '--' in argv:
        i = argv.index('--')
        argv, extra = argv[:i], argv[i + 1:]
    binaries = [os.path.abspath(b) for b in (argv[0] if argv else 'target/compact/zfetch').split(',')]
    runs = int(argv[1]) if len(argv) > 1 else 30
    times: dict[str, list[float]] = {b: [] for b in binaries}
    rss: dict[str, list[int]] = {b: [] for b in binaries}
    for b in binaries:
        wall_ms(b, extra)  # warm-up
    for _ in range(runs):
        for b in binaries:  # interleaved = paired runs
            times[b].append(wall_ms(b, extra))
            rss[b].append(rss_kib(b, extra))
    for b in binaries:
        print(f'{b}: runs={runs} avg_rss_kib={statistics.mean(rss[b]):.1f} '
              f'median_ms={statistics.median(times[b]):.2f}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
