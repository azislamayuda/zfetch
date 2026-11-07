# Release process

zfetch releases are built by one manual GitHub Actions workflow, **Release**
(`.github/workflows/release.yml`). It builds all nine packages, writes `SHA256SUMS`,
and, only when you ask for a real run, creates the tag and a **draft** release.
Publishing is always done by hand.

## Steps

1. **Bump the version** in three places, using the same `X.Y.Z` everywhere:
   - `Cargo.toml` → `version = "X.Y.Z"` (then run `cargo build` so `Cargo.lock` updates)
   - `packaging/arch/PKGBUILD` → `pkgver=X.Y.Z` (or run `python3 packaging/arch/update-pkgver.py`)
   - `docs/RELEASE-NOTES-vX.Y.Z.md` → the release notes; a real run fails without this file
2. **Commit and push to `main`** through GitHub Desktop.
3. **Dry run**: GitHub → Actions → **Release** → *Run workflow* on branch `main`.
   Enter the version (for example `0.5.0`) and leave **dry_run** checked.
   A dry run builds, tests, and checksums everything and uploads the `release-assets`
   artifact. It creates no tag and no release.
4. **Real run**: when the dry run is green, run the workflow again with the same version
   and **dry_run unchecked**. The workflow:
   - re-checks that tag `vX.Y.Z` and release `vX.Y.Z` do not exist yet,
   - creates an annotated tag `vX.Y.Z` on the commit the run started from,
   - creates a **draft** release titled `zfetch vX.Y.Z` with the notes file and ten files
     (nine packages plus `SHA256SUMS`).
5. **Review and publish**: open Releases → the draft. Check the notes and the assets, and
   download a few assets to confirm `sha256sum -c SHA256SUMS`. Then click *Publish release*.

## Release assets

| File | Built on |
| --- | --- |
| `zfetch_X.Y.Z-1_amd64.deb`, `zfetch_X.Y.Z-1_arm64.deb` | Ubuntu 22.04 container (GLIBC 2.35 baseline) |
| `zfetch-X.Y.Z-1.el9.x86_64.rpm`, `zfetch-X.Y.Z-1.el9.aarch64.rpm` | Rocky Linux 9 container (GLIBC 2.34) |
| `zfetch-X.Y.Z-linux-x86_64-gnu.tar.gz`, `zfetch-X.Y.Z-linux-aarch64-gnu.tar.gz` | Rocky Linux 9 container (GLIBC 2.34) |
| `zfetch-X.Y.Z-linux-x86_64-musl.tar.gz`, `zfetch-X.Y.Z-linux-aarch64-musl.tar.gz` | Ubuntu 24.04 runners, static musl |
| `zfetch-X.Y.Z-1-x86_64.pkg.tar.zst` | Arch Linux container |
| `SHA256SUMS` | Checksums of the nine files above |

arm64/aarch64 packages are built on native GitHub `ubuntu-24.04-arm` runners, so every
binary is smoke-tested on its own architecture.

## If something fails

- **Version mismatch**: the `validate` job names the file that disagrees. Fix it, commit, and rerun.
- **Tag or release already exists**: a real run refuses to continue. Delete the stale draft
  or tag on GitHub only if you are sure it is unused, then rerun.
- **Build job failed after the tag was created**: this cannot happen. The tag is created only
  after all builds and checksums have passed. If `publish` itself fails after pushing the tag,
  delete the tag on GitHub, fix the problem, and rerun.
- Runs for the same version never overlap: the workflow uses a per-version concurrency group.
