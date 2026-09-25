# The Flauz Web Client (WEB-001)

The production-gate web client: a TypeScript + React single-page app
served by `crates/flauz-web-gateway`, consuming the app-server protocol
through the gateway's authenticated WebSocket bridge.

**Contract law (Wave-6 kernel addendum §3):** the app-server protocol
types in `src/protocol/generated.ts` are GENERATED from
`src/protocol/schema.json` (the captured `CODEX_APP_SERVER_SCHEMA_EXPERIMENTAL=1`
export). Hand-written protocol types are forbidden — drift is a contract
violation. The gateway transport envelope lives in
`src/gateway/protocol.ts` (the mirror of the gateway crate's descriptor).

## Layout

```
src/
  protocol/   schema.json (the export artifact) · generated.ts · client.ts (the typed bridge)
  gateway/    the gateway WebSocket envelope (the transport contract mirror)
  state/      the connection-state machine (connection.ts) + the app driver (app.tsx)
  app/        the shell: banner, sign-in, workspace home, new task, session view,
              approvals, command palette, shortcuts overlay
  keyboard/   the keyboard map + focus trap/restoration (the 017 law)
  theme/      light/dark theming (system default + persistence)
  strings/    the i18n-ready string dictionary (en shipped)
lab/          the web parity lab: journeys.mjs (J-01..J-05) + mock-gateway.mjs
scripts/      the schema export + type generation pipeline
```

## Commands

```
npm install                 # dependencies
npm run build               # tsc strict + vite build → dist/ (served by the gateway)
npm test                    # vitest: the connection-state machine, the banner, the bridge client
npm run protocol:export     # capture the app-server schema export (needs the codex binary:
                            #   CODEX_RS_CODEX_BIN=<path> npm run protocol:export)
npm run protocol:generate   # regenerate src/protocol/generated.ts from the snapshot
npm run lab:journeys        # run the J-01..J-05 headless journey lab (see below)
```

## Running against the gateway

```
cargo run -p flauz-web-gateway -- --web-root web/dist
# then open http://127.0.0.1:8610/
```

## The journey lab

`npm run lab:journeys` runs the five WEB-001 journeys headless against
the REAL web build and writes evidence (screenshots + action logs +
truthful-state assertions) to `docs/research/evidence/web/`:

- **J-01** Start any project — sign in, empty state, objective → live
  session, palette fallback.
- **J-02** Understand what the agent knows — the Context inspector.
- **J-03** Recover/continue — kill the gateway → named disconnection →
  restart → reconnect → state recovers.
- **J-04** Discover a capability gap — a named error with cause +
  recovery, the unlock path named.
- **J-05** Add another environment/site — the honest foundation empty
  state with the named next step.

By default the lab uses the Node mock gateway (`lab/mock-gateway.mjs` —
identical protocol contract) so it runs anywhere Node can. At the
integration station, run it against the real Rust gateway:

```
node lab/journeys.mjs --gateway "target/debug/flauz-web-gateway --web-root web/dist"
```
