# Internal Model V1

`nex-editor` should not let the runtime's plain-text milestone freeze the wrong core architecture.

This document defines the intended internal editor model for the project.
It references ProseMirror's architecture, but it is not a direct copy.
The goal is to keep the same strengths:

- explicit document structure
- stable position semantics
- transaction-driven editing
- clear separation between document state and rendering

At the same time, v1 stays intentionally narrow enough to power the current minimal editor milestone.

## Design Goal

The current product milestone is a minimal plain-text editor.
That does not mean the internal core should be "just a string".

The internal model should already support:

- `doc -> paragraph -> text` structure
- stable positions and selections
- transaction-based edits
- a clean runtime facade for layout and platform events

The internal model should not yet optimize for:

- rich-text UI features
- pagination
- collaboration
- plugin-driven extension APIs
- IME-complete editing semantics

## Layering

The intended dependency direction is:

`model -> transform -> state -> commands -> runtime -> wasm -> shells`

Responsibilities:

- `model`
  - schema, nodes, marks, fragments, resolved positions
- `transform`
  - steps, mappings, replace semantics
- `state`
  - editor state, selections, transactions
- `commands`
  - user-facing editing actions built on transactions
- `runtime`
  - event interpretation, layout, hit testing, render snapshots
- `wasm`
  - serialization and bridge only
- shells
  - native event capture and drawing only

## Core Model

### 1. Schema Model

The schema layer defines which document shapes are valid.

Minimum v1 schema:

- `doc`
- `paragraph`
- `text`

This is enough to power the current plain-text milestone while preserving a future path toward structured editing.

Required concepts:

- `Schema`
- `NodeType`
- `MarkType`
- `ContentExpr`
- `ContentMatcher`

Rules:

- Shells never consume the schema directly.
- Runtime does not invent document structure outside the schema.
- Even when the UI behaves like a notepad, document text still lives in nodes.

### 2. Document Node Model

The document itself should be represented as typed nodes, not as a runtime-owned `String`.

Minimum node hierarchy:

- `doc`
  - `paragraph`
    - `text`

Required concepts:

- `Node`
- `Fragment`
- `Mark`

Current code already contains a `Node` system, but the runtime still edits by flattening to `String` and rebuilding the document.
That is acceptable only as a temporary bridge.

Target rule:

- edits apply to document structure through steps and transactions
- plain-text extraction is a derived view, not the editing source of truth

### 3. Position Model

Position semantics are the most important ProseMirror idea to preserve.

The editor core should not let runtime code casually manipulate string indices without document context.

Required concepts:

- `ResolvedPos`
- `SelectionRange`
- `Selection`
- `TextSelection`
- `NodeSelection`

Position model requirements:

- all selections must be resolvable against a document
- commands and transforms should operate on positions, not ad hoc shell offsets
- runtime hit testing may produce offsets, but those offsets should still map into document positions

For the minimal editor milestone:

- `TextSelection` is the primary active selection type
- `NodeSelection` can remain structurally present but lightly used

### 4. Transform Model

Document changes should be represented as transform steps.

Required concepts:

- `Step`
- `StepResult`
- `StepMap`
- `Mapping`

Minimum useful step set:

- insert text
- delete range
- replace range
- set selection

This matches the current plain-text scope while preserving a path to richer operations later.

Rules:

- runtime should not directly mutate document content
- commands should produce transactions
- transactions should accumulate mappings so selections and follow-up operations stay coherent

### 5. State Model

`EditorState` is the authoritative in-memory editor state.

Required fields:

- `doc`
- `selection`
- `stored_marks`

Deferred fields:

- plugin state
- history state
- collaboration state

Rules:

- layout data does not belong inside `EditorState`
- shell state does not belong inside `EditorState`
- `EditorState` must stay platform-agnostic

### 6. Runtime Model

`runtime` is not the document model.
It is the facade that turns platform events into editing actions and layout snapshots.

Input boundary:

- `EditorEvent`

Output boundary:

- `RenderSnapshot`

Runtime responsibilities:

- interpret pointer and keyboard events
- map them to commands or transactions
- perform layout and hit testing
- expose render-ready geometry to shells

Runtime non-responsibilities:

- owning the source-of-truth document format
- inventing a parallel string-based editing model
- embedding web-only behavior

### 7. Layout Model

Layout is a derived structure produced from editor state.

Minimum useful layout concepts:

- `LineLayout`
- `TextRun`
- `CaretLayout`
- `SelectionRect`

Possible future additions:

- block layout tree
- inline fragments
- bidi runs
- glyph metrics
- viewport clipping metadata

Rule:

- layout is derived from `EditorState`
- layout is not the canonical document structure

## Current Gap

Today the repository is between two architectures:

- the lower crates already resemble a ProseMirror-style editor core
- the runtime still edits by flattening the document to text, applying string operations, and rebuilding a `doc -> paragraph -> text` tree

That means the project currently has:

- a promising structural foundation
- an unstable plain-text editing bridge

The bridge is good enough for the current browser milestone, but it should not become the permanent core.

## V1 Target Shape

The intended v1 editor flow is:

1. shell sends `EditorEvent`
2. runtime interprets it
3. runtime invokes command or transaction logic
4. transaction applies `Step`s to `EditorState.doc`
5. runtime derives layout from updated `EditorState`
6. runtime returns `RenderSnapshot`

In short:

- editing truth lives in `EditorState`
- document truth lives in `Node`
- mutation truth lives in `Transaction` and `Step`
- render truth lives in derived layout structs

## Minimum Document Shape

To support the notepad milestone without blocking future structure, the internal document should standardize on:

```text
doc
  paragraph
    text("hello")
  paragraph
    text("world")
```

Interpretation rules:

- line breaks in plain-text input become paragraph boundaries or explicit soft breaks, but the choice must be made in one place
- runtime layout should consume the canonical document representation, not a second string model
- plain-text export remains available as a convenience projection

For the current milestone, the canonical rule is:

- each newline in the plain-text projection maps to a paragraph boundary in the document model

That means:

- runtime editing may still expose plain-text input and output
- but internally, `"hello\nworld"` is represented as two paragraphs, not one text node containing a newline character

## Stability Plan

What should be stabilized first:

1. minimal schema for `doc / paragraph / text`
2. selection and position invariants
3. transaction-driven text editing
4. runtime event-to-command mapping
5. render snapshot and layout contract

What should stay flexible for now:

- mark semantics
- plugin architecture
- history format
- serialization format for persistence
- binary protocol for shells

## Immediate Refactor Direction

The next internal refactor should move the runtime away from string-owned editing.

Recommended order:

1. define the minimal canonical plain-text document representation
2. route text insertion and deletion through `Transaction`
3. keep `RenderSnapshot` stable while replacing runtime internals
4. remove document rebuild paths that call `Node::from_paragraph_texts(...)` after raw string edits

## Non-Goals

This design does not yet try to specify:

- full ProseMirror plugin compatibility
- DOM-based editing behavior
- clipboard schema conversions
- collaborative rebasing
- pagination rules
- advanced typography

## Summary

`nex-editor` should treat the current minimal editor as a product milestone, not as permission to adopt a string-only core.

The right internal direction is:

- ProseMirror-style document and position model
- transaction-driven editing
- runtime-owned layout and hit testing
- thin shells that only forward events and draw snapshots

That gives the project a clean path from a minimal notepad-like editor to a richer structured editor without rewriting the core twice.
