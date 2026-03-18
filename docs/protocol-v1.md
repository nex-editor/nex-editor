# Cross-Platform Protocol V1

`nex-editor` v1 uses a headless Rust runtime as the editing source of truth.
Platform shells such as the browser playground, desktop apps, or mobile apps
send editor events into Rust and render immutable layout snapshots returned by Rust.

This document defines the first stable shell boundary exposed by
[`crates/runtime/src/lib.rs`](/Users/jc/Desktop/JC/nex-editor/crates/runtime/src/lib.rs).

## Scope

Protocol v1 is intentionally narrow.

- Product goal: plain-text editor behavior similar to a minimal notepad
- Source of truth: Rust `EditorRuntime`
- Layout and hit testing: Rust-owned
- Rendering: shell-owned
- Transport: in-process calls by default, JSON as the debug and integration shape

The purpose of v1 is not to model a full document editor. It is to lock down the smallest useful cross-platform contract for:

- typing text
- deleting text
- moving the caret
- selecting text
- rendering Rust-owned layout in thin shells

Protocol v1 does not define:

- rich-text marks in the shell contract
- pagination
- collaborative editing
- undo/redo
- binary transport

Current implementation note:

- the browser playground now forwards composition lifecycle events
- Rust holds explicit composition preview state
- committed document text still changes only on commit

## Runtime Boundary

The shell boundary is:

- editor events into Rust
- render snapshots out of Rust

Current Rust types:

```rust
pub struct RenderSnapshot {
    pub text: String,
    pub display_text: String,
    pub selection_anchor: usize,
    pub selection_head: usize,
    pub revision: u64,
    pub composition: Option<CompositionSnapshot>,
    pub viewport: Viewport,
    pub content_width: f32,
    pub content_height: f32,
    pub scene: SceneSnapshot,
    pub lines: Vec<LineLayout>,
    pub selection_rects: Vec<SelectionRect>,
    pub caret: Option<CaretLayout>,
}

pub enum EditorEvent {
    ResizeViewport { width: f32, height: f32, device_pixel_ratio: f32 },
    SetTextMetrics { font_family: String, font_size_px: f32, char_width: f32, line_height: f32, caret_width: f32, ascent: f32, descent: f32 },
    SetTextMeasurements { entries: Vec<TextMeasurementEntry> },
    CompositionStart,
    CompositionUpdate { text: String },
    CompositionEnd { text: String },
    CompositionCancel,
    PointerDown { x: f32, y: f32, button: PointerButton, modifiers: Modifiers, click_count: u8 },
    PointerMove { x: f32, y: f32, modifiers: Modifiers },
    PointerUp { x: f32, y: f32, button: PointerButton, modifiers: Modifiers },
    InsertText { text: String },
    Backspace,
    DeleteForward,
    MoveCaretLeft,
    MoveCaretRight,
    MoveCaretUp,
    MoveCaretDown,
    SelectAll,
}
```

## `RenderSnapshot`

The product boundary still returns `RenderSnapshot`, but shell rendering should
now primarily consume `scene`.

See [Render Protocol V1](/Users/jc/Desktop/JC/nex-editor/docs/render-protocol-v1.md).

### Fields

`text`

- Full plain-text document contents.
- Newlines are represented as `\n`.
- In the current internal model, each `\n` corresponds to a paragraph boundary in the document tree.
- Included for debugging, clipboard-style integrations, and shell inspection.

`display_text`

- Current visible text projection used for layout and painting.
- During normal editing it matches `text`.
- During IME composition it includes preedit text that is not yet committed to the document.
- Shell renderers should treat this as the authoritative visible string for the current frame.

`selection_anchor`

- One end of the current selection.
- When equal to `selection_head`, the selection is collapsed and represents the caret.
- Offsets are expressed in canonical plain-text character positions, not UTF-8 byte offsets.

`selection_head`

- The active end of the current selection.
- Shells may use `min(anchor, head)` and `max(anchor, head)` to render the selected range.
- Uses the same character-offset coordinate system as `selection_anchor`.

`revision`

- Monotonic counter incremented only when document text changes.
- Selection-only changes do not increment it.
- Intended for shell cache invalidation and debugging, not as a globally unique version id.

`composition`

- Current composition/preedit range owned by Rust.
- `null` when no composition session is active.
- The current implementation exposes `{ from, to, text }`.
- Stable direction in v1; field shape may grow if shells later need richer IME metadata.

`viewport`

- Viewport used for the current layout pass.
- Rust uses this to compute lines, hit testing, selection rectangles, and caret geometry.

`content_width`

- Width of the laid out content in shell coordinate space.
- Intended for shell scroll containers and viewport decisions.

`content_height`

- Height of the laid out content in shell coordinate space.
- Intended for shell scroll containers and viewport decisions.

`scene`

- Explicit render-facing scene derived by Rust.
- Shells should prefer this over reconstructing a scene from compatibility fields.
- Includes a style table, background rects, selection rects, text runs, and caret geometry.
- Stable direction in v1.

`lines`

- Ordered layout lines with positioned runs.
- Retained for debugging and migration.
- Shells should not treat this as the preferred long-term drawing API.

`selection_rects`

- Precomputed selection highlight rectangles in shell coordinate space.

`caret`

- Precomputed caret rectangle in shell coordinate space.
- `null` when the selection is non-collapsed.

### Invariants

- `selection_anchor <= text.chars().count()`
- `selection_head <= text.chars().count()`
- `revision` is non-decreasing for a single runtime instance
- line, selection, and caret geometry are internally consistent and ready to render as-is

### Stability

Stable in v1:

- field names
- field meanings
- newline encoding as `\n`
- selection and caret offsets are plain-text character offsets
- revision behavior for text edits vs. selection-only changes
- shells do not perform their own hit testing or line wrapping

May evolve after v1:

- richer run metadata
- scroll metrics
- richer caret and selection metadata

Internally, runtime layout is now expected to flow through explicit layout services
such as text layout and hit-test results, rather than ad hoc helper chains.

Shells should ignore unknown future fields when consuming JSON snapshots.

## `EditorEvent`

### `ResizeViewport { width, height, device_pixel_ratio }`

Update the shell viewport used for layout.

Behavior:

- reflows lines in Rust
- does not modify text
- does not increment `revision`

Stable in v1.

### `SetTextMetrics { font_family, font_size_px, char_width, line_height, caret_width, ascent, descent }`

Update the text measurement facts supplied by the shell.

Behavior:

- shell measures the active render font environment
- shell provides ascent/descent so Rust can compute baseline-based vertical geometry
- Rust updates layout inputs from those measured values
- Rust still owns line wrapping, hit testing, selection geometry, and caret geometry
- does not modify text
- does not increment `revision`

Stable direction in v1.

### `SetTextMeasurements { entries }`

Update cached text advances supplied by the shell.

Each entry currently contains:

- `style_key`: shell measurement style identity
- `text`: a measured grapheme cluster
- `advance`: measured horizontal advance in shell coordinate space

Behavior:

- shell measures text using the active platform font stack for the referenced style
- shell may send entries incrementally as new grapheme clusters appear
- Rust caches those advances by `style_key`
- Rust still owns line wrapping, hit testing, caret geometry, and selection geometry
- does not modify text
- does not increment `revision`

Stable in v1:

- event name
- `style_key + text -> advance` cache model
- `text` is expected to be a user-visible grapheme cluster, not a UTF-8 byte slice

May evolve after v1:

- richer style descriptors
- shaped cluster ids
- vertical metrics attached to style keys

### `CompositionStart / CompositionUpdate / CompositionEnd / CompositionCancel`

Drive IME/preedit state from the shell into Rust.

Behavior:

- shell forwards composition lifecycle events from the platform input host
- Rust owns composition preview state
- preview text may appear in `display_text` and render scene output before commit
- committed document text changes only on `CompositionEnd`
- `revision` increments only when committed text changes

### `PointerDown / PointerMove / PointerUp`

Pointer events in shell coordinate space.

Behavior:

- Rust performs hit testing
- primary-button drag updates selection in Rust
- shells should not translate coordinates into text offsets
- does not increment `revision`

Stable in v1.

### `InsertText { text }`

Insert `text` at the caret, or replace the current selection with `text`.

Behavior:

- replaces the selected range if selection is non-collapsed
- otherwise inserts at the caret
- places the caret after inserted text
- increments `revision` if text changed

Stable in v1.

### `Backspace`

Delete backwards.

Behavior:

- if selection is non-collapsed, delete the selected range
- otherwise delete the code unit immediately before the caret
- otherwise delete the previous plain-text character
- if already at offset `0`, no-op
- collapses selection at the deletion point
- increments `revision` only if text changed

Stable in v1.

### `MoveCaretLeft`

Move the caret or collapse the selection to its start.

Behavior:

- collapses non-collapsed selection to the lower bound
- otherwise moves the caret left by one offset
- does not increment `revision`

Stable in v1.

### `MoveCaretRight`

Move the caret or collapse the selection to its end.

Behavior:

- collapses non-collapsed selection to the upper bound
- otherwise moves the caret right by one offset
- does not increment `revision`

Stable in v1.

### `MoveCaretUp`

Move the caret to the previous visual line.

Behavior:

- collapses non-collapsed selection to its lower bound before moving
- uses Rust-owned visual line layout, not shell-side line math
- keeps the closest available visual column on the target line
- does not increment `revision`

Stable in v1.

### `MoveCaretDown`

Move the caret to the next visual line.

Behavior:

- collapses non-collapsed selection to its upper bound before moving
- uses Rust-owned visual line layout, not shell-side line math
- keeps the closest available visual column on the target line
- does not increment `revision`

Stable in v1.

### `DeleteForward`

Delete forwards.

Behavior:

- if selection is non-collapsed, delete the selected range
- otherwise delete the code unit immediately after the caret
- if already at the end, no-op
- collapses selection at the deletion point
- increments `revision` only if text changed

Stable in v1.

### `SelectAll`

Select the whole document.

Behavior:

- sets selection to `0 -> text.len()`
- does not modify text
- does not increment `revision`

Stable in v1.

## JSON Shape

For browser/WASM debugging and shell integration, v1 snapshots are currently
serialized to JSON.

Example:

```json
{
  "text": "hello\nworld",
  "selection_anchor": 3,
  "selection_head": 3,
  "revision": 2,
  "viewport": { "width": 900.0, "height": 480.0, "device_pixel_ratio": 1.0 },
  "content_width": 96.0,
  "content_height": 84.0,
  "lines": [],
  "selection_rects": [],
  "caret": { "x": 52.8, "y": 27.0, "width": 2.0, "height": 22.0 }
}
```

Guidance:

- field names are stable in v1
- shells should decode and render snapshots as-is rather than derive their own layout
- shells should treat this as the canonical debug shape
- future binary transport must preserve the same semantic contract

## Shell Responsibilities

Shells own:

- focus
- key and pointer event capture
- painting text, selection, and caret

Rust owns:

- document text
- selection state
- edit application
- line wrapping
- coordinate-to-text-offset hit testing
- selection and caret geometry
- clamping and no-op semantics

Shells must not:

- treat local DOM state as the editing truth
- mutate text outside `EditorEvent`
- infer revision changes without reading snapshots
- perform their own selection hit testing or wrapping logic

## Compatibility Notes

For desktop or mobile shells, prefer this migration order:

1. call `EditorRuntime` directly in-process
2. mirror `EditorEvent` and `RenderSnapshot` exactly
3. let Rust own layout and hit testing
4. only then consider richer render snapshots or binary transport

If protocol v2 is introduced later, this document should remain the source of
truth for v1 compatibility behavior.
