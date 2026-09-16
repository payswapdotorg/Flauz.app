# Codex ↔ Flauz Current Version Delta

> **Status: SKELETON.** Historical layer populated from repo records.
> Current layer (ChatGPT desktop `26.825.51511`) is **PENDING Worker A**
> (see `docs/research/evidence/codex-ref/version-delta-notes.md`).

**Repository:** `payswapdotorg/Flauz.app`
**Owner of the current-layer research:** Worker A (CODEX-REFERENCE-MATRIX
wave); reconciliation into parity rows: Worker C2.
**Feeds:** `docs/research/CODEX-FLAUZ-FEATURE-PARITY-REPORT.md` (§2 Reference
layers, §7.7 Version-skew handling).

---

## 1. Method note — two reference layers

The parity mission compares against **two retained reference layers**:

```text
Historical baseline: Codex Desktop 26.721.3996.0 + CLI 0.146.0-alpha.3.1
        +
Current installed:   ChatGPT desktop 26.825.51511 (operator's machine)
        =
Current parity target
```

- The **historical baseline** is the pinned behavioral specification this
  repo was built against; its evidence (in-repo captures, manifest,
  parity-matrix rows) is retained and remains valid for that version.
- The **current installed reference** (`26.825.51511`) defines where the
  parity bar stands *today*: the current target is the historical baseline
  plus the delta introduced since.
- Historical evidence is never discarded in favor of the current layer; rows
  are annotated when the delta moves the bar (parity report §7.7).

## 2. Historical layer — what `26.721.3996.0` is (from repo records)

The historical baseline is the locally installed official Windows package
**`OpenAI.Codex_26.721.3996.0_x64__2p2nqsd0c76g0`**, fingerprinted in
`reference/stable-26.721.3996.0/manifest.json` (captured 2026-07-24): the
closed-source Electron app (`ChatGPT.exe`, `chrome.dll`,
`resources/app.asar`), the bundled CLI (`resources/codex.exe`), runtime
`owl` / Chromium `150.0.7871.128`, and file SHA-256 hashes.
`reference/README.md` states the directory stores metadata only and that
this manifest "fingerprints the locally installed build used as the
executable specification."

Repo records pinning this baseline:

| Record | Statement | Citation |
| --- | --- | --- |
| README | "The behavior reference is Codex Desktop `26.721.3996.0`, which bundled Codex CLI `0.146.0-alpha.3.1`" | `README.md` (lines 63–64) |
| Architecture | "codexRS uses Codex Desktop `26.721.3996.0` and its bundled Codex CLI `0.146.0-alpha.3.1` as an executable behavioral specification" | `docs/architecture.md` (lines 5–6) |
| Platform support | "The stable compatibility oracle is Windows Codex Desktop `26.721.3996.0` with Codex CLI `0.146.0-alpha.3.1`" | `docs/platform-support.md` (lines 12–13) |
| Agent guidance | "Treat stable `26.721.3996.0` as a behavioral reference, not a runtime" | `AGENTS.md` (line 5) |
| Parity matrix | Reference baseline table: Windows package, desktop bundle build `5828` / internal `26.721.31836`, bundled CLI `0.146.0-alpha.3.1` (+SHA-256), stable app-server schema 89 / experimental 126 client request methods, UI inspection date 2026-07-25 | `docs/parity-matrix.md` §Reference baseline |
| Manifest | Package identity, runtime, per-file hashes | `reference/stable-26.721.3996.0/manifest.json` |
| Code | `stable_reference()` pins `package_version: "26.721.3996.0"`, `cli_version: "0.146.0-alpha.3.1"` (asserted in tests) | `crates/codex-core/src/lib.rs` (~line 223) |
| Historical failure oracle | Stable failure modes + active release-candidate limitations recorded against this baseline | `docs/known-failures.md` |

Historical-layer bound: this baseline is a **Windows package**; its UI
evidence is `[historical-record]` (app.asar inspection 2026-07-25) and the
official runtime is reproducible in this lab only through the official CLI
`0.146.0-alpha.3.1` (`codexrs probe` — see
`docs/research/E2B-PARITY-ENVIRONMENT.md`, branch `parity/lab`).

## 3. Current layer — ChatGPT desktop `26.825.51511`

**PENDING Worker A** — see
`docs/research/evidence/codex-ref/version-delta-notes.md` (to be delivered by
Worker A with the CODEX-REFERENCE-MATRIX wave).

This section will hold, once Worker A delivers:

- what `26.825.51511` is and how it was observed (provenance-labeled);
- the version-range delta `26.721.3996.0 → 26.825.51511` in official
  release notes `[docs-derived]`;
- the runtime/protocol delta signal: upstream `openai/codex` releases
  `rust-v0.146.0-alpha.3.1 → rust-v0.154.0` (latest) as already recorded in
  `docs/research/E2B-PARITY-ENVIRONMENT.md` §Linux-A `[docs-derived]`;
- a per-area list of behaviors that changed between the two layers, feeding
  parity-report version-skew annotations (§7.7 there).

Until Worker A delivers, **no parity row may cite the current layer**; the
historical rows stand as the pre-delta baseline.

## 4. Change log

| Date | Change |
| --- | --- |
| 2026-09-16 | Skeleton created; historical layer populated from repo records; current layer marked PENDING Worker A (Worker C1). |
