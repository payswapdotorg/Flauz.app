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

Journey battery (J-01..J-18 runner) lands as `journeys.py` (LAB-002).
