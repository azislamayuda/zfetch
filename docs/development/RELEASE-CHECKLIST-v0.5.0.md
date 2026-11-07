# Release checklist — zfetch v0.5.0

Prepared 2026-11-08. Nothing on this list was executed by the assistant except where it says
"verified locally". Every release action belongs to the owner.

| # | Item | Status |
| --- | --- | --- |
| 1 | All review findings resolved, or accepted by the owner in writing (`SPRINT-05-REVIEW.md`) | **OPEN**: 0 blockers; 2 majors need a decision (display source, GNOME version source); 8 minors and 5 nits await accept/fix |
| 2 | Quality gate passes on the **committed** tree | **PENDING**: verified locally on the uncommitted working tree (all 8 steps PASS); re-run after commit (CI covers this) |
| 3 | Owner tests on GNOME Ubuntu: `zfetch`, `--doctor`, `--benchmark`, `--privacy`, `--format json` | **PENDING**: the assistant ran these on the owner's Ubuntu 26.04 GNOME laptop with an isolated config only; the owner must run them with the real config and logo |
| 4 | Owner tests on every other distro claimed (Fedora/Rocky, Arch, Alpine, Gentoo, Void, Hyprland/Sway) | **PENDING**: only Ubuntu 24.04 (container) and 26.04 were tested |
| 5 | Owner commits to `main` through GitHub Desktop, and origin has the commit | **DONE** (2026-11-08, committed and pushed by the assistant at the owner's request); `baseline-v0.4.0/` left untracked; `PROJECT_CONTEXT.md` not staged |
| 6 | All five workflows (CI, Debian, RPM/GNU portable, musl portable, Arch) green for that commit | **PENDING**: not triggered |
| 7 | One combined `SHA256SUMS` covering all nine packages attached, and verified with `sha256sum -c` | **PENDING**: `--update` needs the combined `SHA256SUMS` asset (v0.4.0 has one) |
| 8 | Tag `v0.5.0` created by the owner | **PENDING**: owner only |
| 9 | Release notes finalized from `docs/RELEASE-NOTES-v0.5.0.md`; README asset table and install commands switched to 0.5.0 file names | **PENDING** |
| 10 | `--update` tested from v0.4.0 to v0.5.0 on a Debian machine, with the config left intact | **PENDING**: only possible after the v0.5.0 release is public (v0.4.0's updater finds v0.5.0); check that `~/.config/zfetch/config.jsonc` and the logos are unchanged afterwards |

**Overall: not ready to start.** There are no blockers, but item 1 has two open majors that need the owner's decision.
