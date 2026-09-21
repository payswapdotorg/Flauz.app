# REL-RC14 — Release v0.1.0-rc.14: parity hardening

**Status:** COMPLETE
**Release:** `v0.1.0-rc.14` (tag at `ddc614e`, workspace version bumped in the
same commit)
**Predecessor:** `v0.1.0-rc.13` (2026-09-16) — see `GUI-006-report.md`

## What shipped

The parity hardening release: the full `v0.1.0-rc.13..v0.1.0-rc.14` range —
P1 discoverability (Terminal/Browser surfaces, multi-folder projects), the
P2 parity batch (004 settings-palette indexing, 005 slash subset, 006 side
chats, 007 Ctrl+P routing, 008 unread attention, 009 history rows, 010
palette rows with guards, 011 browser chords, 012 guard honesty, 013
Activity view, 018 visible entries/labels, 020 honest no-workspace status),
WO-UX-002 (keyboard-only chat creation), WO-P2-019 (confirmation-modal
focus traps), and WO-P2-017 (palette/overlay close focus restoration with
the r1 close-path panic eliminated). Twelve `Added` + five `Fixed`
user-facing changelog entries.

## Publish record

The `Release` workflow on tag `v0.1.0-rc.14` (run **35585925384**, all four
jobs green):

- **Third-party licenses** — `THIRD_PARTY_LICENSES.html` (cargo-about),
  dependency policy.
- **Package windows-x86_64** — full gates (fmt, patched-renderer
  clippy/test, workspace clippy `-D warnings`, workspace tests, release
  build), Windows ZIP packaging, archive startup smoke.
- **Package linux-x86_64** — same gates, tar.gz packaging, isolated Xvfb
  startup smoke.
- **Publish GitHub release** — `SHA256SUMS.txt` + release notes.

Assets (3): `codexrs-v0.1.0-rc.14-windows-x86_64.zip` (16,289,607 B),
`codexrs-v0.1.0-rc.14-linux-x86_64.tar.gz` (18,998,196 B),
`SHA256SUMS.txt`. Marked prerelease (rc tag). Title: "Flauz.app
v0.1.0-rc.14".

## Fresh-machine validation

`scripts/fresh_machine_validate.sh` executed against the published release
(`RELEASE_TAG=v0.1.0-rc.14`, pinned Codex runtime, headless-sandbox library
shims via `EXTRA_LIBS` — the GUI-006 posture, nothing on the host modified):

1. **Download** — both artifacts fetched from the published release.
2. **Checksum verify** — `sha256 OK:
   9eb1139caeca6c6b9fec6c968715f7c715c4fa509dffa42609ef73b0002505d4`
   (linux tar.gz matches `SHA256SUMS.txt`).
3. **Install (portable archive)** — extracted; `codexrs` executable present.
4. **Bounded runtime information** — `codexrs info` prints the bootstrap
   banner (references, runtime, protocol limits) and exits 0.
5. **Fresh HOME + XDG + CODEX_HOME; desktop integration** —
   `--install-desktop-entry` writes
   `com.codexrs.CodexRS.desktop` with an `Exec=` line into the isolated
   `XDG_DATA_HOME`.
6. **Runtime probe (fresh machine)** — app-server initialized;
   codex-home configured (verified); threads 0.
7. **Restart (second probe on the same fresh home)** — app-server
   initialized again; idempotent.

Result: **FRESH_MACHINE_VALIDATION_OK**.
