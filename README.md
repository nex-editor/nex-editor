# nex-editor

`nex-editor` is being built as a headless Rust editor runtime with thin platform shells.

The current first milestone is intentionally narrow: a minimal plain-text editor that behaves like a notepad.
Before adding rich-text structure, paging, or advanced rendering, the project is stabilizing:

- Rust-owned editing state
- Rust-owned layout and hit testing
- a small cross-platform event/render protocol
- a browser playground that only forwards events and draws snapshots

## Web Playground

The repository now includes a minimal WASM-driven plain text playground at `apps/web-playground`.

```bash
cd apps/web-playground
npm install
npm run dev
```

That flow builds the WASM package from `crates/wasm`, writes it into `apps/web-playground/src/wasm`, and starts Vite.
The playground shell forwards browser events into Rust and renders Rust-produced layout snapshots on canvas.

Today the playground target is only:

- click to place the caret
- type text
- delete text
- make selections
- move the caret with arrow keys
- render the result from Rust-owned layout data

Not in the current milestone:

- rich-text marks
- document schema UX beyond plain text
- pagination
- collaboration
- IME/composition completeness

## Protocol

The current shell/runtime contract is documented in
`docs/protocol-v1.md`.
