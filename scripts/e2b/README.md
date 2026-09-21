# E2B Desktop journey harness (Linux GUI lane)

Primary Linux UX test environment tooling. See
`docs/linux-gui/LINUX-GUI-E2B-ENVIRONMENT.md` for the environment record and
reproducibility contract.

- `flauz_e2b.py` — sandbox lifecycle, reproducible provisioning (apt deps,
  rustup 1.97.1, pipewire 1.0.5 side-install for Ubuntu 22.04-class images —
  lane finding L-001, Flauz clone @ pinned commit, release build), GUI
  launch, real user interaction primitives, evidence capture.
- Credentials are read from the Lead station environment
  (`~/.secrets/env.sh`: `E2B_API_KEY`, `PAYSWAP_PAT`) — never committed,
  logged, or placed in evidence.

Run (Lead station):

```bash
python3 scripts/e2b/flauz_e2b.py provision   # idempotent; pins current main
python3 scripts/e2b/flauz_e2b.py launch      # start the real GUI on the desktop
python3 scripts/e2b/flauz_e2b.py info        # environment record
```

## Journey battery (LAB-002)

`journeys.py` is the J-01..J-18 regression battery on top of the harness
above (provisioning incl. the L-001 pipewire side-install and the automatic
L-002 gpui visual patch, launch, interaction). Specs are declarative
(`journey_specs.py`, coordinates pinned to run-3/run-4 pixel-verified
evidence); every journey runs a cold-start primary pass and a palette pass;
every step captures a screenshot + assertion results into
`docs/linux-gui/evidence/<run-id>/` and a run section is appended to the
[journey matrix](../../docs/linux-gui/LINUX-GUI-USER-JOURNEY-MATRIX.md).

```bash
python3 scripts/e2b/journeys.py --dry-run       # anywhere: plan + spec validation, no E2B
python3 scripts/e2b/journeys.py --battery baseline                    # Lead station (E2B_API_KEY in env)
python3 scripts/e2b/journeys.py --journeys J01,J05 --battery baseline # subset
python3 scripts/e2b/journeys.py --run-id <id> --battery baseline      # resume after pause/expiry
python3 scripts/e2b/journeys.py --battery baseline --baseline <prev-run-id>  # screenshot regression
python3 scripts/e2b/tests.py                    # unit tests incl. FakeSandbox full battery
```

Flags: `--battery` (baseline = full sweep), `--journeys` subset,
`--dry-run`, `--run-id`, `--commit` (pin provision; default main HEAD),
`--baseline` (compare vs a previous run), `--repo-root` (testing).
Pillow (optional, Lead station) enables region probes and the calibrated
palette differential; without it those assertions degrade honestly.

