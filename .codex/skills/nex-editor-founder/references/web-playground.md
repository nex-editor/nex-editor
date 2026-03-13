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

`npm run build` should produce a complete browser bundle, including WASM output under `public/wasm`.

## Browser Constraints

- Canvas is the current render surface.
- The browser owns focus, mouse events, key events, and local text layout.
- Rust owns document state, selection state, and edit application.

## When Editing The Playground

- Keep the render loop simple and inspectable.
- Prefer explicit coordinate-to-offset logic over hidden DOM selection behavior.
- Add browser-only features only if they help validate the runtime contract.
- Do not bury core editor semantics inside TypeScript helpers.
