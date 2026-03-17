# web-playground

Minimal browser playground for the headless runtime.

## Notes

- The runtime boundary is `EditorEvent -> RenderSnapshot`.
- Generate the browser package into `apps/web-playground/src/wasm` so Vite can treat the WASM bridge as part of the app module graph.
- Rust owns editing, layout, hit testing, selection geometry, and caret geometry.
- Canvas owns drawing only; the browser shell forwards events and does not use `contenteditable` as source of truth.
- The current product target is a minimal plain-text editor, not a rich-text document editor.
- Keep playground changes focused on validating typing, deletion, selection, caret movement, and layout ownership.
- Keep `main.ts` as a thin shell composer. DOM shell mounting, WASM protocol access, canvas rendering, and browser event mapping should live in separate modules.
- Run `npm run test:e2e` to verify real browser interaction with the canvas editor.
