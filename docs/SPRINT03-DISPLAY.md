# zfetch v0.3.0 — Sprint 03: Layout & Theme Refinement

## Scope

- Introduce `--layout compact|classic|detailed` (terminal mode only).
- `classic` remains the default, retaining existing logo selection, custom MJ logo, alignment, headings, spacing and all configured modules.
- `compact` renders selected information in a clean vertical list using the existing colors and label formatting, omitting the ASCII logo and decorative headings. It does **not** reduce the number of modules by itself; combine it with `--preset minimal` or `--only`.
- `detailed` uses the classic renderer and automatically enables `--hardware-details` for selected modules. This can incur additional sysfs reads. It does not activate during normal execution.
- Improve `--list-themes` with actual ANSI named-color sample blocks on a TTY (plain output under pipes or `NO_COLOR`). Theme application remains `--theme default|monochrome|nord|gruvbox|rust|high-contrast`.
- Fix GPU name compression to retain verified hardware information appended with ` | ` (e.g., `integrated | Current 600 MHz | Max 1300 MHz`). Unrelated PCI ID brackets remain hidden.
- Add CLI parser and GPU formatter regression tests.

## Usage

```bash
zfetch                                 # unchanged classic output with your MJ config
zfetch --layout compact
zfetch --layout compact --preset minimal
zfetch --layout classic --theme nord
zfetch --layout detailed --theme rust
zfetch --layout detailed --only cpu,gpu,display,battery
zfetch --list-themes
zfetch --format json --only cpu,gpu --hardware-details
```

JSON `--format json` intentionally rejects `--layout compact` and `--layout detailed`, which control **terminal presentation**, not machine-readable output. Use `--hardware-details` with JSON to request extended metadata. `--layout classic` is allowed alongside JSON (no effect on JSON).

## Compatibility & boundaries

- Source files are modified only after all patch anchors pass a precheck.
- The installer preserves `.config/zfetch/`, custom logo files, distro ASCII logo database, and update manager.
- No dependency additions, network fetch, `git commit`, `git push`, or package installation.
- No automatic terminal width probing is included in this Sprint; that is a potential future enhancement after end-to-end testing.
- Themes continue to use zfetch's 8 ANSI named colors, not true-color palette support.
- Benchmarks and correctness across other distros still require verification.

## Validation

```bash
cargo fmt --all -- --check
cargo check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --profile compact
./target/compact/zfetch --layout compact --preset minimal
./target/compact/zfetch --layout detailed --only cpu,gpu,display,battery
./target/compact/zfetch --list-themes
./target/compact/zfetch --format json --only cpu,gpu --hardware-details | python3 -m json.tool
```

If any check fails, use backups from `target/backups/pre-sprint03-*` or review the changes in GitHub Desktop. Do not revert unrelated changes.

Suggested GitHub Desktop commit message after all tests pass:

`feat: add terminal layout modes and theme previews (v0.3.0 sprint 03)`
