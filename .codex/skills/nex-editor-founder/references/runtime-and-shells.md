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

## Current Runtime Flow

Inside Rust, the current runtime flow is:

1. `EditorEvent` enters `EditorRuntime::dispatch`
2. dispatch routes to:
   - pointer handling
   - editing orchestration
   - layout-aware navigation
   - viewport updates
3. reusable text-edit semantics call into `commands`
4. commands produce `Transaction`
5. `Transaction::commit()` produces next `EditorState`
6. runtime derives `TextLayout`
7. runtime exposes `RenderSnapshot`

Current runtime submodules:

- `commands.rs`: runtime-to-commands bridge
- `editing.rs`: editing orchestration
- `pointer.rs`: pointer interaction handling
- `navigation.rs`: layout-aware vertical navigation
- `layout.rs`: text layout, hit testing, caret/selection geometry

Guardrails:

- do not let `dispatch` accumulate real editing logic
- do not reintroduce naked `usize` offset plumbing when `FlatTextOffset` is the real semantic type
- do not put document mutation logic in layout code
- do not move runtime behavior back into shells

## Preferred Evolution

1. Stabilize plain-text event and render-snapshot flow
2. Keep pulling reusable edit semantics into `commands`
3. Keep layout semantics shared in Rust across browser, desktop, and mobile shells
4. Add richer selection/navigation behavior at the runtime boundary
5. Introduce binary transport only after the snapshot shape is stable

## Cross-Platform Principle

- Browser, desktop, and mobile should share the Rust editing truth
- Platform differences should live in input plumbing and rendering
- Avoid platform-specific behavior forks in lower Rust layers
