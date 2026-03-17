# Architecture Constraints V1

This document defines the short-term architectural constraints that should guide
`nex-editor` while the project is still building its minimal plain-text editor.

The purpose is simple:

- prevent local implementation shortcuts from becoming permanent architecture
- clarify which boundaries are already stable
- identify which simplifications are intentional and allowed

This document is stricter than product docs and narrower than long-term vision docs.

## Scope

These constraints apply to:

- `crates/model`
- `crates/transform`
- `crates/state`
- `crates/commands`
- `crates/runtime`
- `crates/wasm`
- thin shells such as `apps/web-playground`

## Constraint 1: Editing Truth Lives In Rust

The canonical editor state lives in Rust.

Shells must not own:

- document state
- selection state
- line wrapping decisions
- hit testing rules
- caret geometry

Shells may own:

- native focus
- native event capture
- drawing
- shell-local UI state that does not change editor semantics

## Constraint 2: The Runtime Boundary Stays Narrow

The stable runtime boundary is:

- input: `EditorEvent`
- output: `RenderSnapshot`

`wasm` is only a bridge.
It must not become a second runtime layer with editor semantics hidden inside bindings.

## Constraint 3: Document State Is Not A String

The product milestone is a minimal plain-text editor.
The internal core is still document-based.

Allowed:

- plain-text projection of the document
- plain-text export in snapshots
- plain-text-oriented layout for the current milestone

Not allowed:

- making a runtime-owned `String` the source of truth
- adding editor behavior that bypasses `EditorState`
- duplicating document state in shells

## Constraint 4: Canonical Plain-Text Projection Is Explicit

The current minimal editor needs a plain-text projection.
That projection must remain explicit and centralized.

Current rule:

- `doc` paragraphs join with `\n`
- each `\n` in the projection corresponds to a paragraph boundary

This projection is a compatibility layer for the current milestone.
It is not permission to blur the distinction between document positions and plain-text offsets.

## Constraint 5: Offsets And Positions Must Not Be Mixed Implicitly

There are currently two useful coordinate spaces:

- document-aware positions
- plain-text offsets used by layout and shell interaction

The project may continue using plain-text offsets in runtime-facing layout code for now.
But these offsets must be treated as an explicit type and conversion boundary.

Rules:

- avoid passing naked `usize` values when the semantic meaning is "plain-text offset"
- hit testing should return a typed result, not a bare integer
- selection and command code should make conversion boundaries visible

## Constraint 6: Commands Own Editing Semantics

`commands` should be the main home for reusable editing behavior.

`runtime` may still orchestrate:

- pointer-driven selection changes
- viewport-dependent vertical movement
- layout-derived navigation

But `runtime` should not accumulate document-editing logic that can live in `commands`.

Current direction:

- insert text belongs in `commands`
- delete backward belongs in `commands`
- delete forward belongs in `commands`
- select all belongs in `commands`

## Constraint 7: Runtime Is A Facade, Not A Dumping Ground

`runtime` should be split by responsibility:

- event routing
- pointer handling
- editing orchestration
- layout and hit testing

It is acceptable for these parts to share a crate.
It is not acceptable for them to collapse back into one large implementation blob.

## Constraint 8: Layout Is Derived State

Layout is derived from editor state.
It must not become a second document representation.

Allowed layout structures:

- visual lines
- runs
- caret geometry
- selection rectangles
- text layout services
- hit-test results

Not allowed:

- mutating document semantics inside layout code
- shell-specific behavior mixed into layout services

## Constraint 9: The Minimal Document Schema Is Fixed For Now

Until the minimal editor milestone is complete, the working schema constraints are:

- `doc`
- `paragraph`
- `text`

Assumptions that runtime and commands may rely on for now:

- paragraph is the only active block type in the minimal editor path
- marks may exist in the model but are not yet a primary product surface
- shells do not need direct schema awareness

If these assumptions change, the architecture docs and affected code must change together.

## Constraint 10: Platform Shells Stay Thin

For web, desktop, and mobile:

- shells forward events
- shells receive render snapshots
- shells draw

Do not move editor behavior into shells just because it is faster to prototype there.

## Immediate Priorities

The most important architectural follow-ups are:

1. keep pulling reusable edit semantics into `commands`
2. keep layout-aware navigation separate from generic editing orchestration
3. keep layout services separate from document mutation logic
4. avoid reintroducing naked `usize` offset plumbing in runtime code

## Non-Goals

These constraints do not yet define:

- plugin APIs
- collaborative rebasing
- pagination
- binary runtime transport
- advanced typography
- full IME model

## Summary

For the current milestone, `nex-editor` is allowed to be simple.
It is not allowed to be vague.

The project should remain:

- Rust-owned for editing truth
- document-based internally
- explicit about plain-text projection boundaries
- command-driven for editing semantics
- layout-driven for rendering and hit testing
- thin-shelled across platforms
