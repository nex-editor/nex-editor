# Architecture

## Layer Order

`model -> transform -> state -> commands -> runtime -> wasm/shells`

Keep responsibilities strict:

- `model`: immutable document data, schema, positions
- `transform`: atomic document changes and mapping
- `state`: editor state, selection, transaction commit
- `commands`: user-intent helpers over transactions
- `runtime`: product-facing plain-text facade
- `wasm/shells`: transport and rendering shells

## Current End-To-End Flow

The current minimal editor should be understood as this pipeline:

1. shell captures a native event
2. shell maps it to `EditorEvent`
3. `runtime` routes the event
4. editing events delegate to `commands` where possible
5. `commands` build `Transaction`s over `EditorState`
6. `Transaction::commit()` produces the next `EditorState`
7. `runtime` derives `TextLayout`
8. `runtime` builds render-facing `SceneSnapshot`
9. `runtime` packages `RenderSnapshot`
10. shell draws the snapshot

Practical interpretation:

- document truth lives in `model` and `state`
- editing semantics should keep moving into `commands`
- `runtime` should orchestrate event routing, pointer handling, layout-aware navigation, layout, scene construction, and snapshots
- `wasm` should stay a bridge, not a behavior layer

## Design Defaults

- Start from the narrowest schema that supports the current product goal.
- Prefer behavior-level APIs over leaking internal tree structure.
- If a new feature is reusable editing semantics, prefer `commands` before `runtime`.
- If a new feature is layout-aware or pointer-aware orchestration, keep it in `runtime`.
- Pagination is a derived layout concern, not a model concern.

## Implementation Standard

- `Step::apply` must produce a real document change or fail.
- `Transaction::commit()` must build the next state from steps, not from shortcuts.
- Selection mapping is part of correctness, not optional polish.
- Tests should assert visible behavior, not only internal helper calls.
