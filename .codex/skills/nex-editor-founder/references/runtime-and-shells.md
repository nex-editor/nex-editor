# Runtime And Shells

## Product Boundary

The stable shell contract is:

- input commands into Rust
- snapshots out of Rust

Current default boundary:

- `EditorRuntime`
- `EditorSnapshot`
- `InputCommand`

## Shell Rules

- Shells do layout and drawing unless there is a clear reason to move layout into Rust.
- Shells should not inspect internal document nodes when the snapshot already contains enough information.
- Keep the first shell protocol easy to debug. JSON is acceptable before binary transport is justified.

## Preferred Evolution

1. Stabilize plain-text snapshot and command flow
2. Add richer selection/navigation behavior
3. Add richer render snapshots only when multiple shells need the same layout semantics
4. Introduce binary transport only after the snapshot shape is stable

## Cross-Platform Principle

- Browser, desktop, and mobile should share the Rust editing truth
- Platform differences should live in layout, input plumbing, and rendering
- Avoid platform-specific behavior forks in lower Rust layers
