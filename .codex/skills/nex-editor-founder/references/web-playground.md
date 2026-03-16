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
- The browser owns focus, mouse events, and key events.
- Rust owns document state, edit application, layout, hit testing, selection geometry, and caret geometry.

## When Editing The Playground

- Keep the render loop simple and inspectable.
- Do not add browser-side coordinate-to-offset logic when Rust can own hit testing.
- Add browser-only features only if they help validate the runtime contract.
- Do not bury core editor semantics inside TypeScript helpers.
- When fixing browser interaction bugs, add or update an E2E scenario that reproduces the issue.
