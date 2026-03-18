# Current Flow V1

This document describes the current execution flow of `nex-editor` for the
minimal plain-text editor milestone.

It is not the long-term architecture vision.
It is the practical flow that the codebase follows today.

## End-To-End Path

The current browser flow is:

1. browser captures a native event
2. browser shell maps it to `EditorEvent`
3. WASM bridge forwards the event into Rust
4. `EditorRuntime::dispatch` routes the event
5. runtime delegates to:
   - pointer handling
   - editing orchestration
   - layout-aware navigation
   - viewport updates
6. reusable edit semantics flow into `commands`
7. `commands` build `Transaction`
8. `Transaction::commit()` produces the next `EditorState`
9. runtime derives `TextLayout`
10. runtime builds `RenderSnapshot`
11. runtime packages a render-facing `scene` for shells
12. WASM returns JSON snapshot
13. browser shell renders the snapshot on canvas
14. browser shell measures missing grapheme advances for the active text style
15. browser shell sends `SetTextMeasurements`
16. Rust reflows layout with the cached advances

## Runtime Flow

Inside Rust, the current runtime is organized like this:

- [lib.rs](/Users/jc/Desktop/JC/nex-editor/crates/runtime/src/lib.rs)
  - protocol types
  - runtime state
  - top-level event routing
- [commands.rs](/Users/jc/Desktop/JC/nex-editor/crates/runtime/src/commands.rs)
  - bridge from runtime orchestration into the `commands` crate
- [editing.rs](/Users/jc/Desktop/JC/nex-editor/crates/runtime/src/editing.rs)
  - editing orchestration and transaction application
- [pointer.rs](/Users/jc/Desktop/JC/nex-editor/crates/runtime/src/pointer.rs)
  - pointer-driven selection handling
- [navigation.rs](/Users/jc/Desktop/JC/nex-editor/crates/runtime/src/navigation.rs)
  - layout-aware vertical movement
- [layout.rs](/Users/jc/Desktop/JC/nex-editor/crates/runtime/src/layout.rs)
  - text layout
  - hit testing
  - selection rectangles
  - caret geometry

## Data Flow

The main data transitions are:

1. `EditorEvent`
2. `Transaction`
3. `EditorState`
4. `TextLayout`
5. `SceneSnapshot`
6. `RenderSnapshot`

This means:

- editor mutation happens through `Transaction`
- runtime layout is derived from committed editor state
- shell rendering is derived from `RenderSnapshot`

## Browser Shell Flow

The current web playground shell is organized like this:

- [shell.ts](/Users/jc/Desktop/JC/nex-editor/apps/web-playground/src/shell.ts)
  - mount DOM shell
  - expose `canvas`, `status`, `revision`
- [input.ts](/Users/jc/Desktop/JC/nex-editor/apps/web-playground/src/input.ts)
  - map browser input to protocol events
- [metrics.ts](/Users/jc/Desktop/JC/nex-editor/apps/web-playground/src/metrics.ts)
  - measure active browser font metrics
  - cache grapheme advances keyed by `style_key`
- [bridge.ts](/Users/jc/Desktop/JC/nex-editor/apps/web-playground/src/bridge.ts)
  - initialize WASM bridge
  - exchange JSON render snapshots and debug snapshots
- [debug.ts](/Users/jc/Desktop/JC/nex-editor/apps/web-playground/src/debug.ts)
  - render shell-local debugging panels
  - show serialized Rust document JSON and recent operation logs
- [renderer.ts](/Users/jc/Desktop/JC/nex-editor/apps/web-playground/src/renderer.ts)
  - draw `snapshot.scene` to canvas
- [status.ts](/Users/jc/Desktop/JC/nex-editor/apps/web-playground/src/status.ts)
  - format debug/status output
- [main.ts](/Users/jc/Desktop/JC/nex-editor/apps/web-playground/src/main.ts)
  - compose shell modules

## Current Ownership Model

Rust owns:

- document state
- selection state
- commands and transactions
- layout
- hit testing
- caret geometry
- selection geometry
- render-facing scene construction

Shell owns:

- focus
- native pointer events
- native keyboard events
- native text measurement
- canvas drawing
- shell-local operation logging
- debug panel rendering

Rust also exposes a development-only debug snapshot so shells can inspect the
current document tree without becoming responsible for editor logic.

## Current Simplifications

The current flow intentionally assumes:

- minimal document schema: `doc -> paragraph -> text`
- plain-text projection joins paragraphs with `\n`
- text offsets are canonical plain-text character offsets
- line layout is Rust-owned, but shell-supplied text measurement can refine per-grapheme advances
- browser transport uses JSON through WASM bindings

These are current implementation constraints, not permanent product claims.

## What This Flow Is Optimizing For

The current flow is designed to optimize for:

- a stable editor boundary
- a minimal working editor
- shared behavior across future shells
- low ambiguity about where logic belongs

## Near-Term Follow-Ups

The most likely next architectural improvements are:

1. keep moving reusable edit semantics into `commands`
2. grow `TextLayout` into a clearer layout service boundary
3. keep runtime routing thin and explicit
4. preserve thin shells across browser, desktop, and mobile
