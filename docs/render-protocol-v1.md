# Render Protocol V1

This document defines the rendering-facing snapshot boundary for `nex-editor`.

The goal is simple:

- Rust owns editing and layout
- shells own drawing
- shells do not infer editor geometry on their own

## Why This Exists

`EditorEvent -> RenderSnapshot` is the product boundary.

Inside that output boundary, the rendering contract still needs its own shape.
If shells read ad hoc fields and decide for themselves how to reconstruct the
scene, the architecture becomes unstable quickly.

Render Protocol V1 fixes that by making the Rust output explicitly scene-shaped.

## Current Direction

The runtime now exposes a render-facing scene inside `RenderSnapshot`:

```rust
pub struct RenderSnapshot {
    pub text: String,
    pub display_text: String,
    pub selection_anchor: usize,
    pub selection_head: usize,
    pub revision: u64,
    pub viewport: Viewport,
    pub content_width: f32,
    pub content_height: f32,
    pub scene: SceneSnapshot,
    pub lines: Vec<LineLayout>,
    pub selection_rects: Vec<SelectionRect>,
    pub caret: Option<CaretLayout>,
}

pub struct SceneSnapshot {
    pub viewport: Viewport,
    pub content_width: f32,
    pub content_height: f32,
    pub styles: Vec<PaintStyle>,
    pub background: Vec<PaintRect>,
    pub selection_rects: Vec<PaintRect>,
    pub composition_underlines: Vec<PaintRect>,
    pub text_runs: Vec<PaintTextRun>,
    pub caret: Option<PaintRect>,
}
```

The long-term intent is:

- shells render from `scene`
- older debug-oriented fields remain temporarily for inspection and migration

The current measurement direction is also explicit:

- Rust owns layout decisions
- shells provide measured text advances keyed by style identity
- Rust consumes those measured advances and emits final geometry

## Stable Responsibilities

Rust owns:

- text layout
- line wrapping
- text run positions
- text baselines
- selection geometry
- caret geometry
- composition preview state
- content extents

Shells provide:

- measured text metrics for the active font environment
- measured grapheme advances keyed by text style
- viewport facts

Shells own:

- actual paint API calls
- device pixel ratio setup
- viewport host/container wiring
- mapping Rust style tokens onto platform theme values
- font binding

Shells must not own:

- hit testing from pointer position to document offset
- line wrapping decisions
- caret x/y calculation
- selection rectangle derivation

## `SceneSnapshot`

### `viewport`

The viewport used when Rust produced the scene.

Stable in v1.

### `content_width` / `content_height`

Total laid out content size.

Stable in v1.

### `styles`

Render-facing style table for the current scene.

Stable meaning in v1:

- each entry has a stable `id`
- each entry declares a semantic render role
- paint fragments reference styles by `style_id`

This lets shells map semantic roles to concrete platform paint values without
embedding hard-coded values into every fragment.

The same architectural rule applies to text measurement:

- shells provide measured font facts and style-keyed grapheme advances
- Rust consumes those facts and still owns layout decisions

### `background`

Background paint rectangles.

Stable meaning in v1:

- shell may paint these in-order
- current minimal editor uses a single full-viewport background rect
- each rect references a stable `style_id` from the scene style table

May evolve later:

- block backgrounds
- gutter backgrounds
- composition backgrounds

### `selection_rects`

Precomputed selection highlight rectangles.

Stable in v1.

Shells should draw them directly.
Shells should use the provided `style_id` rather than inferring a semantic role
from geometry alone.

### `composition_underlines`

Precomputed preedit/composition underline fragments.

These allow shells to render IME preview state without owning composition
layout logic.

### `text_runs`

Ordered text fragments ready to render.

Stable meaning in v1:

- each run already has x/y/width/height
- each run references a stable `style_id`
- each run exposes `baseline_y`
- shell does not recompute wrapping
- shell does not merge or split runs for correctness

May evolve later:

- font/style ids
- bidi direction
- decoration metadata

### `caret`

Optional caret paint rect.

Stable in v1:

- `null` when selection is non-collapsed
- otherwise ready to render directly
- when present, includes a `style_id`

## Style Table

Render Protocol V1 now includes a render-facing style table.

Current semantic roles are:

- `EditorSurface`
- `PrimaryText`
- `SelectionFill`
- `CaretFill`
- `CompositionUnderline`

Rust chooses which `style_id` applies to each fragment and associates that
style id with a semantic role.
Each shell maps those roles to platform-native paint values.

For text styles, the scene may also expose `measurement_style_key`.
This gives shells a stable measurement namespace without making them the owner
of layout or run positioning.

This preserves the architecture split:

- Rust owns semantic render intent
- shells own concrete theme values and drawing APIs

Measurement rule:

- shells measure text against `measurement_style_key`
- shells send `SetTextMeasurements`
- Rust caches those advances and reflows

## Temporary Compatibility Fields

These fields still exist on `RenderSnapshot`:

- `lines`
- `selection_rects`
- `caret`

They remain useful for:

- debugging
- migration
- tests that inspect intermediate layout structure

They should be treated as compatibility/debug fields, not the long-term shell
rendering API.

## Recommended Shell Rule

Shell renderers should follow this order:

1. paint `scene.background`
2. paint `scene.selection_rects`
3. paint `scene.text_runs`
4. paint `scene.caret`

This order is stable for the minimal editor milestone.

Shell renderers should draw text fragments using an alphabetic baseline model.
For the current protocol shape, `baseline_y` is the authoritative vertical text
anchor.

## Near-Term Evolution

The next likely changes are:

1. expand the style table into richer text and decoration styling
2. add block fragments and decoration fragments
3. move more shell implementations to consume only `scene`
4. eventually retire compatibility layout fields from shell-facing usage
