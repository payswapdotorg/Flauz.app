# REL-001 — Completion Report

**Work order:** REL-001 — the signed-release scaffold: SHA256SUMS manifest +
the GPG-ready signing step + the archive verification mode + the Linux
distribution strategy doc (Phase: Production, Wave 8, item 1 of the trio,
dispatched Wave-8 Worker A).

**Base branch + SHA:** `main` @
`4563d0f5a1ba10d5bb7b444055b19eefa5f6461b` (the packet's STEP ZERO pin;
verified `git rev-parse HEAD` = `4563d0f5a1ba10d5bb7b444055b19eefa5f6461b`
immediately after clone+checkout, and `origin/main` equals the same SHA at
clone time). Note: the work-orders doc's dispatch-notes line records
`af2210d` — that line was written before the work-orders doc commit itself
landed; `4563d0f` is exactly that doc commit sitting directly on top of
`af2210d` on main, so the packet's explicit STEP ZERO command governs and
matches main.

**Branch/commits:** `feat/rel-001-signing`, one clean commit:
`feat(rel): REL-001 — the signed-release scaffold: SHA256SUMS + GPG-ready signing step + verify_release_archive.sh + the Linux distribution strategy (Wave 8)`
Delivery bundle: `rel-001-delivery.bundle`
(`4563d0f5a1ba10d5bb7b444055b19eefa5f6461b..feat/rel-001-signing`), exported
at the repo root. NOT pushed (the Lead harvests the bundle).

**RE-ENTRY LAW check:** the REL-001 artifacts did NOT exist at the pinned
base (`scripts/verify_release_archive.sh` absent; no signing steps in
`release.yml`; no distribution-strategy section in `docs/platform-support.md`
— verified at STEP ZERO). Full authoring dispatch, not verify-and-report.

## Changed files / surfaces

- `.github/workflows/release.yml` (MODIFIED — ADDITIVE steps only,
  **+78/−0**): two new steps in the `publish` job, inserted between the
  pre-existing "Create checksums" step and "Publish release":
  - `Validate checksum manifest coverage (REL-001)` — the always-on gate
    that hard-fails the publish when any packaged archive in `dist/`
    (`./*.zip`, `./*.tar.gz`) lacks a `SHA256SUMS.txt` entry (acceptance
    criterion 1 enforced at run time; fail-closed on an empty artifact set).
  - `Sign release checksums (GPG-ready, REL-001)` — the detached-signature
    step over `SHA256SUMS.txt` producing `SHA256SUMS.txt.asc` (published by
    the existing `dist/*` glob in `gh release create`, unchanged):
    conditionally SKIPPED with a NAMED log line when the
    `FLAUZ_RELEASE_SIGNING_KEY` secret is absent (exit 0 — never fails the
    build, never fakes a signature); when the key is present it imports it
    into an ephemeral `mktemp -d` GNUPGHOME via stdin (never echoed — the
    credential law), signs with the key's fingerprint, round-trip-verifies
    the produced signature, and hard-fails with named errors on
    gpg-absent/import-failure/no-secret-key/failed-round-trip. Optional
    `FLAUZ_RELEASE_SIGNING_KEY_PASSPHRASE` supported via `--passphrase-fd 3`
    (never on argv). The pre-existing "Create checksums" step is kept
    untouched (see the honest note below).
- `scripts/verify_release_archive.sh` (NEW, 100644 like the other
  `scripts/` shell entries): verifies one local archive against a
  SHA256SUMS file + the detached signature when one is present (explicit
  third argument, or auto-detected `<manifest>.asc` / `<manifest>.sig`).
  Signature first (authenticity of the oracle), then the checksum entry
  (integrity of the download). Hard-fails with NAMED messages:
  `SIGNATURE-VERIFY-FAILED`, `VERIFY-TOOLING-ABSENT` (gpg or
  sha256sum/shasum missing — a present signature is never silently
  downgraded to checksum-only), `MANIFEST-MALFORMED`,
  `MANIFEST-ENTRY-MISSING`, `MANIFEST-DUPLICATE-ENTRIES`,
  `CHECKSUM-MISMATCH`, `VERIFY-INPUT-MISSING`, usage. Exit codes: 0 pass,
  1 verification failure, 2 usage/input/environment. Parses GNU
  `sha256sum` text and binary separator forms, tolerates CRLF manifests,
  normalizes the workflow's `./`-prefixed entry names, and falls back to
  `shasum -a 256` when `sha256sum` is absent. Prints the named
  `SIGNATURE-ABSENT` honest-bound line when no signature exists
  (checksums detect corruption; they do not authenticate the publisher).
- `docs/platform-support.md` (EXTENDED — ADDITIVE section, **+51/−0**): new
  `## Linux distribution strategy` section (between `## Linux` and
  `## Runtime directories`): the channel table with status + recovery path
  for every channel (tar.gz+SHA256SUMS = Supported; detached signature =
  Scaffolded with the operator provisioning path; AppImage = evaluation
  with the honest no-AppImage-toolchain-in-CI bound + recovery; `.deb` =
  evaluation with the no-deb-packaging-path/no-apt-repo bound + recovery;
  Snap/Flatpak = not evaluated; source build = Supported), the
  manifest-and-archives-share-a-channel honesty paragraph, and the
  `### Verifying a downloaded archive` instructions (the script,
  `sha256sum --check --ignore-missing`, and `gpg --verify`).
- `docs/research/evidence/rel-001/REL-001-COMPLETION-REPORT.md` (NEW — this
  file; the packet's reporting contract requires the 11-field report ON the
  branch; see Contract deviations).

**Zero product-source changes:** `git diff --stat` vs the pinned base shows
only the surfaces above — `2 files changed, 129 insertions(+)` on tracked
files plus the two NEW files (the script and this report). No crate, no
THIRD-PARTY/LICENSE/README surface, no `ci.yml`, no packaging-list change
(the work order's "packaging lists stay as-is" holds: the packaged file set
is untouched; `docs/platform-support.md` was already packaged and its
extension rides along).

**Honest note (pre-existing checksum step):** the work order's Problem says
the pipeline "produces no checksum manifest". At the pinned base the
`publish` job already ran `sha256sum ./*.zip ./*.tar.gz > SHA256SUMS.txt`
(step "Create checksums", present since `8b95dc9` "Prepare native
v0.1.0-rc.1"). REL-001 does not duplicate or rewrite it (additive-only
contract); instead the new coverage gate ENFORCES exactly the work order's
requirement — one SHA256SUMS entry per packaged artifact, verified against
the actual `dist/` contents at run time, fail-closed. Acceptance criterion
1 is satisfied by the kept generator + the new always-on gate.

## Implementation summary

The work order's "Checksums + signing" unit maps to the publish-job chain:
`download-artifact (both matrix legs, merged) → Create checksums (kept,
generates SHA256SUMS.txt) → Validate checksum manifest coverage (REL-001)
(new gate) → Sign release checksums (GPG-ready, REL-001) (new) → Publish
release (uploads dist/*, so SHA256SUMS.txt and — once produced —
SHA256SUMS.txt.asc ride along)`. The integration note is honored: the
checksum work runs AFTER both package steps (publish needs the whole build
matrix) and BEFORE the upload (`gh release create`); the matrix labels
(`windows-x86_64`, `linux-x86_64`) are untouched (asserted by the YAML
gate). The honest bound is exactly the addendum §2 pattern: no signing key
exists in this environment, so the step skips with the named
`[REL-001] SIGNING-SKIPPED` line and the recovery path (operator provisions
`FLAUZ_RELEASE_SIGNING_KEY`) is stated in the log line, the workflow
comment, the strategy doc, and here — no signature is ever faked, and the
skip never fails the build.

## Tests / commands + exact results

Worker-sandbox battery (gpg 2.4.7, GNU coreutils sha256sum, Python 3 +
PyYAML 6.0.3 available; the battery itself lives OUTSIDE the repo tree):

1. `bash -n scripts/verify_release_archive.sh` — **exit 0** (T1).
2. `python3` `yaml.safe_load` on `.github/workflows/release.yml` — **PASS**:
   publish step order asserted as `actions/download-artifact@v8 → Create
   checksums → Validate checksum manifest coverage (REL-001) → Sign release
   checksums (GPG-ready, REL-001) → Publish release`; matrix labels
   `windows-x86_64, linux-x86_64` intact; both new step `run:` bodies
   extracted from the YAML and executed verbatim against fixtures (so the
   tested code IS the shipped workflow text) (T2).
3. Script self-test — the work order's "checksum a fixture, tamper it,
   verify the failure" plus the full matrix, **31/31 PASS, 0 FAIL**:
   - clean verify: exit 0, `[REL-001] CHECKSUM-VERIFIED … VERIFY-PASSED` +
     the `SIGNATURE-ABSENT` honest line (T3/T3b);
   - tampered archive: exit 1, `CHECKSUM-MISMATCH: … expected 9ae24193…,
     got 24df4e6c…` (T4);
   - tampered manifest hash: exit 1 `CHECKSUM-MISMATCH` (T5); missing
     entry: exit 1 `MANIFEST-ENTRY-MISSING` (T6); hex-valid duplicate
     entry: exit 1 `MANIFEST-DUPLICATE-ENTRIES` (T7); malformed line: exit
     1 `MANIFEST-MALFORMED` (T8);
   - usage: exit 2 with usage line (T9); missing archive / manifest /
     explicit signature: exit 2 `VERIFY-INPUT-MISSING` (T10–T12);
   - signature present + gpg absent: exit 2 `VERIFY-TOOLING-ABSENT`
     (no silent downgrade) (T13);
   - real-GPG flow: scratch ed25519 key (T14); valid signature verifies —
     exit 0 `SIGNATURE-VERIFIED` (T15); manifest tampered after signing:
     exit 1 `SIGNATURE-VERIFY-FAILED` (T16); valid signature + tampered
     archive: exit 1 `CHECKSUM-MISMATCH` (T17); `.sig` suffix auto-detect
     (T18); CRLF manifest still verifies (T19);
   - workflow bodies: coverage gate passes on a complete fixture —
     `[REL-001] checksum manifest covers every packaged artifact:
     ./codexrs-…-windows-x86_64.zip ./codexrs-…-linux-x86_64.tar.gz` (T20);
     an uncovered extra artifact hard-fails — `::error::[REL-001]
     SHA256SUMS.txt does not cover every packaged artifact` + diff, exit 1
     (T21); signing step with the key absent (empty secret, as GitHub
     renders it): exit 0, the named `[REL-001] SIGNING-SKIPPED: …`
     + `::warning::` line, and NO `SHA256SUMS.txt.asc` produced (T22/T22b);
     with a provisioned key: `[REL-001] SIGNATURE-PRODUCED:
     SHA256SUMS.txt.asc (signing key A0732EDB…) — published with the release
     assets` + the file exists (T23/T23b); a third party holding ONLY the
     public key verifies it with `gpg --verify` exit 0 (T24); a
     passphrase-protected key signs via `--passphrase-fd 3` exit 0 (T25);
     garbage key material: exit 1, `::error::[REL-001] gpg failed to import
     FLAUZ_RELEASE_SIGNING_KEY …` (T26);
   - credential law: grep over every signing-step log output for
     `BEGIN PGP PRIVATE KEY BLOCK` — zero hits (T27);
   - full publish-chain simulation (create-checksums → coverage gate →
     signing skip): exit 0 with the named skip line (T28).

Honest absence: the worker sandbox cannot execute the GitHub Actions
workflow end-to-end (no runner) — the step bodies were therefore extracted
from the committed YAML and executed verbatim; the Lead's workflow-parse +
CI re-run remains the authoritative pipeline gate per addendum §5.

## Kernel-compliance checklist (Wave-8 addendum §1–§5)

- **§1 production hardening never weakens the verified state:** the
  workflow diff is +78/−0 (no existing step, name, glob, matrix label, or
  condition changed — asserted programmatically in T2); zero product-source
  changes, so no verified surface is touched and the synchronization law
  does not trigger.
- **§2 honest bounds over theater:** the named gap (no signing key in this
  environment) carries its recovery path everywhere it appears (workflow
  comment + log line, strategy doc, this report); the skip path exits 0
  with a greppable named line; nothing fakes a signature or a pass
  (invalid key material hard-fails — T26; a present signature that does not
  verify hard-fails — T16; gpg-absent hard-fails rather than downgrades —
  T13).
- **§3 the credential law is absolute:** the signing key is imported from
  the secret through stdin into an ephemeral GNUPGHOME (`2>/dev/null`,
  exit-code-checked), the passphrase travels on fd 3 (never argv), and no
  log line, script, or doc contains key material (T27 grep evidence). The
  verify script contains no credentials by construction.
- **§4 one bounded commit-branch delivery:** pinned full SHA, STEP ZERO
  clone+checkout+branch (rev-parse verified), RE-ENTRY law checked (fresh
  authoring), one clean commit + `rel-001-delivery.bundle` at the repo
  root, this 11-field report, honest absences named.
- **§5 the Lead executes the verification passes:** this report gates, it
  does not close — the Lead re-runs the workflow YAML parse, the
  strategy-doc gates, and CI on the harvested branch.

## Acceptance-criteria evidence (point by point)

1. **SHA256SUMS generated for every packaged artifact in the workflow:** the
   publish job generates `SHA256SUMS.txt` over every `.zip`/`.tar.gz`
   (pre-existing "Create checksums" step, kept), and the new
   `Validate checksum manifest coverage (REL-001)` gate hard-fails the
   publish if any archive in `dist/` lacks an entry — T20 (complete
   manifest passes with the named line) + T21 (uncovered artifact fails
   closed, exit 1).
2. **The signing step skips with a NAMED log line when the key is absent
   (never fails the build, never fakes a signature):** T22 — exit 0 with
   `[REL-001] SIGNING-SKIPPED: the release signing key secret
   FLAUZ_RELEASE_SIGNING_KEY is absent in this environment — no detached
   signature is produced (none is faked; the build is not failed). …
   Recovery path: the operator provisions the FLAUZ_RELEASE_SIGNING_KEY
   secret and re-runs this release workflow.` + the `::warning::`
   annotation; T22b — no `.asc` produced. (Corresponding real-signing
   paths proven live: T23/T24/T25; invalid key fails honestly: T26.)
3. **verify_release_archive.sh verifies + hard-fails on tamper:** T3 (clean
   pass), T4 (tampered archive → exit 1 `CHECKSUM-MISMATCH` with expected
   vs actual hashes), T5 (tampered manifest → exit 1), T16 (post-signing
   manifest tamper → exit 1 `SIGNATURE-VERIFY-FAILED`), T17 (valid
   signature + tampered archive → exit 1) — plus the defensive failures
   T6/T7/T8/T9–T13.
4. **The strategy doc names every channel with its status + recovery
   path:** `docs/platform-support.md` → `## Linux distribution strategy`:
   the six-row channel table (Portable tar.gz + SHA256SUMS = Supported;
   Detached OpenPGP signature = Scaffolded, not yet produced, operator
   provisioning recovery; AppImage = Not shipped — evaluation, no
   AppImage toolchain in CI, toolchain+packaging-step+smoke recovery;
   .deb = Not shipped — evaluation, no deb packaging path in CI and no apt
   repository, cargo-deb/dpkg-deb + apt-repo decision recovery;
   Snap/Flatpak = Not evaluated; Source build = Supported) + the
   same-channel honesty paragraph + the verification instructions.

## Known limitations

- No signing key exists (the named gap): signatures are scaffolding until
  the operator provisions `FLAUZ_RELEASE_SIGNING_KEY`; until then
  `SHA256SUMS.txt` is integrity-only (corruption/tamper-vs-manifest
  detection, not publisher authentication) — stated in the strategy doc,
  the release-notes text (pre-existing, untouched), and the script's
  `SIGNATURE-ABSENT` line.
- The signature covers the manifest (`SHA256SUMS.txt.asc`), not each
  archive individually — standard practice; the manifest's entries cover
  every archive transitively.
- The worker sandbox cannot run the Actions workflow itself (named above);
  the tested step bodies are the verbatim YAML `run:` texts.
- `verify_release_archive.sh` is a bash + coreutils/gpg tool: on Windows it
  runs under Git Bash/WSL; no native PowerShell verifier is shipped
  (Windows Authenticode is a declared non-goal).
- A user without gpg cannot verify a present signature (exit 2
  `VERIFY-TOOLING-ABSENT` with the install recovery hint — T13); a user
  without the publisher's public key gets gpg's "No public key" failure
  surfaced inside the named `SIGNATURE-VERIFY-FAILED` message.
- Publishing the signer's public key (docs page / well-known location) is
  intentionally not done this wave — nothing is signed yet; it belongs to
  the key-provisioning follow-up.

## Contract deviations

**NONE.** One disclosed reconciliation, not a deviation: the packet's
reporting contract requires the 11-field report ON the branch, while the
work order's owned-files list names only the three product surfaces + "
NOTHING else". Per the wave evidence convention (every wave since F2:
`docs/research/evidence/<wo>/<WO>-COMPLETION-REPORT.md`; FV-002 stored its
report inside its owned subsystem), this report is committed at
`docs/research/evidence/rel-001/REL-001-COMPLETION-REPORT.md`. No product,
pipeline, packaging-list, THIRD-PARTY/LICENSE/README, or other-worker
surface was touched.

## Follow-up

- **Operator (one-time, unblocks real signing):** provision the
  `FLAUZ_RELEASE_SIGNING_KEY` repository secret (ASCII-armored OpenPGP
  private key; optional `FLAUZ_RELEASE_SIGNING_KEY_PASSPHRASE`) and publish
  the matching public key; the next tag release then publishes
  `SHA256SUMS.txt.asc` with zero workflow changes.
- **Lead:** once signatures go live, consider a release-notes line pointing
  at `SHA256SUMS.txt.asc` (the notes text was deliberately not touched —
  additive-only); re-run the workflow parse + CI on the harvested branch.
- **Future work orders (Wave 8b candidates):** AppImage/deb evaluation per
  the strategy doc's recovery paths; auto-update (UPD-001) can build on the
  checksum manifest; a Windows-native verifier is out of scope until
  Authenticode is scoped.
