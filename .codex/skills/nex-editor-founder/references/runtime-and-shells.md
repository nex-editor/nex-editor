# Runtime And Shells

## Product Boundary

The stable shell contract is:

- editor events into Rust
- render snapshots out of Rust

Current default boundary:

- `EditorRuntime`
- `RenderSnapshot`
- `EditorEvent`

The formal v1 contract should be documented in `docs/protocol-v1.md`.
When changing any field meaning or command behavior, update the code and the protocol document in the same change.

## Shell Rules

- Rust owns editing, layout, hit testing, selection geometry, and caret geometry.
- Shells draw the provided snapshot and forward native input events.
- Shells should not inspect internal document nodes or recompute line wrapping when the snapshot already contains enough information.
- Keep the first shell protocol easy to debug. JSON is acceptable before binary transport is justified.

## Preferred Evolution

1. Stabilize plain-text event and render-snapshot flow
2. Keep layout semantics shared in Rust across browser, desktop, and mobile shells
3. Add richer selection/navigation behavior at the runtime boundary
4. Introduce binary transport only after the snapshot shape is stable

## Cross-Platform Principle

- Browser, desktop, and mobile should share the Rust editing truth
- Platform differences should live in input plumbing and rendering
- Avoid platform-specific behavior forks in lower Rust layers
