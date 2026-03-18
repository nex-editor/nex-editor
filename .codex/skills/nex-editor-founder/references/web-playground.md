# Web Playground

## Purpose

`apps/web-playground` is a fast feedback shell for the headless runtime. It is not the source of truth for editor behavior.

## Workflow

From `apps/web-playground`:

```bash
npm install
npm run build:wasm
npm run dev
```

`npm run build` should produce a complete browser bundle, with the generated WASM bridge emitted under `src/wasm` before Vite bundles it.

For browser interaction validation:

```bash
npm run test:e2e
```

## Browser Constraints

- Canvas is the current render surface.
- The browser owns focus, mouse events, key events, and IME/composition host plumbing.
- The browser also owns native text measurement for the active font environment.
- Rust owns document state, edit application, layout, hit testing, selection geometry, and caret geometry.

Current browser shell module flow:

- `shell.ts`: mount DOM shell and expose elements
- `shell.ts` also provides the hidden native input host used for keyboard and IME events
- `input.ts`: map native browser events to protocol events
- `bridge.ts`: load WASM bridge and exchange JSON render/debug snapshots and events
- `debug.ts`: render development panels for document JSON and operation logs
- `renderer.ts`: draw `snapshot.scene` to canvas
- `status.ts`: format debug/status text
- `main.ts`: compose the shell

## When Editing The Playground

- Keep the render loop simple and inspectable.
- Prefer consuming explicit Rust scene data instead of reconstructing paint state from debug fields.
- Keep platform theme mapping local to the shell: Rust emits a style table plus fragment style ids, and the web renderer maps style roles to canvas fill/text styles.
- Keep text measurement shell-local but protocol-driven: measure by `measurement_style_key`, cache in the shell, send `SetTextMeasurements`, and let Rust reflow.
- Do not add browser-side coordinate-to-offset logic when Rust can own hit testing.
- Add browser-only features only if they help validate the runtime contract.
- Prefer exposing runtime state through explicit debug panels instead of ad hoc console logging.
- Do not bury core editor semantics inside TypeScript helpers.
- Keep `main.ts` close to a startup/composition file, not a behavior file.
- When fixing browser interaction bugs, add or update an E2E scenario that reproduces the issue.
- Keep Playwright specs thin by routing repeated interactions through shared test hooks/harness helpers under `apps/web-playground/tests`.
