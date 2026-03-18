---
layout: home

hero:
  name: "nex-editor"
  text: "Minimal Editor First"
  tagline: Build the smallest correct plain-text editor in Rust first, then grow richer document features on top.
  actions:
    - theme: brand
      text: Current Flow
      link: /current-flow-v1
    - theme: alt
      text: Render Protocol
      link: /render-protocol-v1

features:
  - title: Simple Plain Text First
    details: The current goal is a minimal notepad-like editor with typing, deletion, caret movement, and selection.
  - title: Rust Owns Behavior
    details: Editing state, layout, hit testing, caret geometry, and selection geometry live in Rust.
  - title: Thin Shells
    details: Web, desktop, and mobile shells should only forward native events and draw render snapshots.
---
