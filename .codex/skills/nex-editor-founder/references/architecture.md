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

## Design Defaults

- Start from the narrowest schema that supports the current product goal.
- Prefer behavior-level APIs over leaking internal tree structure.
- If a new feature can live in `runtime` instead of changing lower layers, keep it in `runtime`.
- Pagination is a derived layout concern, not a model concern.

## Implementation Standard

- `Step::apply` must produce a real document change or fail.
- `Transaction::commit()` must build the next state from steps, not from shortcuts.
- Selection mapping is part of correctness, not optional polish.
- Tests should assert visible behavior, not only internal helper calls.
