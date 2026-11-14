# zfetch v0.6.0: Build provenance and signed checksums

Status: **Draft for owner review. Nothing here is final until the owner approves it.**
Workflow references were checked against `.github/workflows/release.yml` at commit `a957407`.

## Sources read

All read on **2026-11-14**:

| Source | URL |
| --- | --- |
| attest-build-provenance README | https://github.com/actions/attest-build-provenance |
| actions/attest README | https://github.com/actions/attest |
| GitHub Docs, artifact attestations for builds | https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations/using-artifact-attestations-to-establish-provenance-for-builds |
| gh manual, `gh attestation verify` | https://cli.github.com/manual/gh_attestation_verify |
| Sigstore, signing blobs | https://docs.sigstore.dev/cosign/signing/signing_with_blobs/ |
| Sigstore, verifying | https://docs.sigstore.dev/cosign/verifying/verify/ |
| cosign-installer README | https://github.com/sigstore/cosign-installer |

Local checks, same day: `gh repo view` reports the repository as `PUBLIC` and not owned by an
organization. The `gh` on the development machine (`gh version 2.46.0`, Ubuntu package) has
**no `attestation` command**.

## What exists today

`release.yml` builds nine assets in `build-deb`, `build-rpm`, `build-musl`, and `build-arch`.
The `checksums` job checks the exact asset set and writes one combined `SHA256SUMS`. `publish`
(the only job with `contents: write`) re-verifies, tags, and creates a **draft** release. The
workflow default is `permissions: contents: read`.

`SHA256SUMS` proves the files match each other. It does not prove who built them: anyone who
can change the release can change both the assets and `SHA256SUMS`. `zfetch --update` trusts
`SHA256SUMS` from the same release (`update.rs`, `sha256_asset`, `expected_checksum`).

## 1. Build provenance

**Current action:** the `attest-build-provenance` README says that from v4 it "is simply a
wrapper on top of `actions/attest`" and that "new implementations should use `actions/attest`".
The design therefore uses **`actions/attest@v4`**.

**Availability:** the README says attestations are available in public repositories on all
current plans. zfetch is public, so this applies.

**How the nine assets get provenance:** one step in a new `attest` job (section 3):

```yaml
- uses: actions/attest@v4
  with:
    subject-path: |
      release-assets/zfetch*
      release-assets/SHA256SUMS
```

- `subject-path` accepts globs and a newline-separated list, up to 1024 subjects (README).
  This gives one attestation with ten subjects: the nine packages plus `SHA256SUMS`.
- Alternative input: `subject-checksums: release-assets/SHA256SUMS`, which reads digests from a
  SHA256SUMS-style file. It is not used here because it would not attest `SHA256SUMS` itself.
- The attestation is signed with the workflow's OIDC identity through Sigstore. Public
  repositories use the public-good Sigstore instance (README), and GitHub stores it on the
  repository's attestations page.

**How a user verifies** (gh manual):

```bash
gh attestation verify zfetch_0.6.0-1_amd64.deb --repo azislamayuda/zfetch \
  --signer-workflow azislamayuda/zfetch/.github/workflows/release.yml
```

`--repo` is the minimum. `--signer-workflow` also pins the workflow file, so an attestation from
another workflow in the same repository is not accepted. Offline verification uses
`--bundle <file>`, optionally with `--custom-trusted-root`. There is no `--offline` flag.

**Limit found:** the Ubuntu-packaged `gh` 2.46.0 on the development machine has no `attestation`
command. Users on distro-packaged `gh` may not be able to verify until they install a newer `gh`.
The gh manual does not say which version added the command, so the minimum version is **not
verified** (open question 4). This is one reason section 2 still recommends a separate signature
that does not need `gh`.

## 2. Signature on `SHA256SUMS`

| | A. GPG-signed `SHA256SUMS` | B. Sigstore/cosign keyless |
| --- | --- | --- |
| Setup | Owner creates a signing key, publishes the public key (`docs/` or repo root) and its fingerprint | Add `sigstore/cosign-installer` and one `cosign sign-blob` step |
| Where signing happens | Either the private key goes into an Actions secret, or the owner signs locally and uploads `SHA256SUMS.asc` to the draft by hand | In CI, with the workflow OIDC token. No secret |
| Key management | Owner must protect, back up, rotate, and possibly revoke the key. Losing it means a key change that every user must notice | None. Short-lived certificate, recorded in the public Rekor log |
| User needs | `gpg` or `gpgv`, usually preinstalled. `gpgv` is present on every APT system | `cosign` binary, usually **not** preinstalled |
| User command | `gpg --verify SHA256SUMS.asc SHA256SUMS`, after importing the key and checking its fingerprint | `cosign verify-blob SHA256SUMS --bundle SHA256SUMS.sigstore.json --certificate-identity <workflow identity> --certificate-oidc-issuer https://token.actions.githubusercontent.com` |
| Protects against | Asset swap on the release by anyone without the private key, including a stolen GitHub token (if the key is kept offline) | Asset swap by anyone who cannot run **this** workflow on **this** ref; the signature names the exact workflow |
| Does not protect against | A stolen private key; a user who takes the key from the same compromised repo | A compromise that can run `release.yml` on `main` (that person can also sign) |
| Fits the CI gate (section 4) | Only if the key is in CI. Manual signing happens after the workflow, so CI cannot gate on it | Yes, naturally |

**Recommendation: B, cosign keyless.** There is no key to lose. It shares the Sigstore trust
root with the attestations in section 1, and it fails inside the workflow, so section 4 can
block the release.

**When A is better:** if the goal is for `zfetch --update` to verify the signature itself.
`gpgv` is already on every APT system (APT uses it for repository signatures), and a public key
can be embedded in the binary. Cosign verification inside zfetch would need either the `cosign`
binary or new crates. If the owner wants updater verification in v0.6.0 or v0.7.0, choose A and
accept the key management.

**Owner must decide:**

1. Cosign (recommended) or GPG, based on whether `--update` should verify signatures.
2. Whether a detached signature is needed at all. Section 1 already attests `SHA256SUMS` as a
   subject. A smaller option is attestations only; the cost is that verification then needs a
   recent `gh`.
3. For cosign: the exact certificate identity users should pin. The usual form for GitHub Actions
   is `https://github.com/azislamayuda/zfetch/.github/workflows/release.yml@refs/heads/main`.
   The Sigstore pages read for this design do **not** document that form. Confirm it in the first
   real run by inspecting the certificate.

## 3. Workflow changes (`release.yml`)

A new job between `checksums` and `publish`:

```yaml
attest:
  name: Attest and sign release assets
  needs: [validate, checksums]
  if: ${{ !inputs.dry_run }}
  runs-on: ubuntu-24.04
  timeout-minutes: 10
  permissions:
    contents: read
    id-token: write          # OIDC token for Sigstore (attest and cosign)
    attestations: write      # store the attestation
    artifact-metadata: write # listed as required by the actions/attest README
  steps:
    - uses: actions/download-artifact@v8
      with: {name: release-assets, path: release-assets}
    - run: cd release-assets && sha256sum -c SHA256SUMS
    - uses: actions/attest@v4
      with:
        subject-path: |
          release-assets/zfetch*
          release-assets/SHA256SUMS
    - uses: sigstore/cosign-installer@v4.1.3
    - run: |
        cd release-assets
        cosign sign-blob --yes --bundle SHA256SUMS.sigstore.json SHA256SUMS
        cosign verify-blob SHA256SUMS --bundle SHA256SUMS.sigstore.json \
          --certificate-identity "<identity from open question>" \
          --certificate-oidc-issuer https://token.actions.githubusercontent.com
    - uses: actions/upload-artifact@v7
      with: {name: release-signature, path: release-assets/SHA256SUMS.sigstore.json, if-no-files-found: error}
```

Changes to existing jobs:

| Job | Change |
| --- | --- |
| `validate`, `build-*`, `checksums` | none |
| `attest` | **new**, permissions above, **no** `contents: write` |
| `publish` | `needs: [validate, checksums, attest]`; also downloads `release-signature` into `release-assets/`; the file count check `-eq 10` becomes `-eq 11`; `contents: write` stays **only** here |
| `summary` | add `attest` to `needs` and to the result table |

Permission notes:

- The GitHub Docs page lists `id-token`, `contents: read`, and `attestations` for binaries. The
  actions/attest README also lists `artifact-metadata: write`, and the README says storage records
  apply only to `push-to-registry` and organization-owned repositories, which zfetch does not use.
  The sources disagree, so the design keeps the README's list. Dropping `artifact-metadata` is an
  open question that the first real run can settle.
- `cosign-installer` itself "does not need any GitHub permission" (README); signing uses
  `id-token: write`.
- Pin `cosign-release` to an exact version in Phase 2. The README example shows `v3.1.4`, but the
  page gives no default.

**Dry runs:** `attest` is skipped when `dry_run` is true, like `publish`, so test builds do not
create public attestations or Rekor entries. The cost is that the signing path is first exercised
on a real run. Since `publish` depends on it, a failure there stops before any tag (section 4).

## 4. Failure behavior

- `publish` lists `attest` in `needs`. With GitHub Actions' default behavior, if `attest` fails,
  `publish` is **skipped**, so no tag is pushed and no draft is created.
- `publish` has no `if: always()`. Its only condition is `!inputs.dry_run`. Phase 2 must keep it
  that way; a later `always()` there would remove this gate.
- `attest` runs its own `sha256sum -c` and a `cosign verify-blob` self-check, so a bad signature
  fails `attest`, not a user.
- Accepted side effect: if `attest` succeeds and `publish` fails later (for example, the tag
  already exists), the attestation and Rekor entry for those exact bytes remain. They describe
  real builds of that commit and cause no harm. A rerun with the same version produces new
  bytes only if the build is not reproducible; the new run attests its own bytes.
- `summary` (`if: always()`) shows `attest` as `failure` or `skipped`, so the reason is visible.

## 5. Not done in this phase

- `zfetch --update` keeps trusting `SHA256SUMS` as today. Verifying the signature in the updater
  depends on owner decision 1.
- User-facing verification steps belong in the README and the v0.6.0 release notes, after the
  owner chooses an option.
- Actions are referenced by tag (`@v4`), like the rest of `release.yml`. Pinning by commit SHA is
  stricter, but it is a repository-wide decision, not part of this change.

## 6. Open questions for the owner

1. Cosign (recommended) or GPG, and should `--update` verify signatures (and in which version)?
2. Attestations only, or attestations plus a detached signature?
3. Keep `artifact-metadata: write`, or drop it after the first real run confirms it is unused?
4. Minimum `gh` version to document for `gh attestation verify` (not verified; the local Ubuntu
   `gh` 2.46.0 lacks the command).
5. Also publish the attestation bundle (`bundle-path` output) as a release asset, so users can
   verify offline with `--bundle`?
6. Should a dry run sign as well, accepting public Rekor entries for test builds?
