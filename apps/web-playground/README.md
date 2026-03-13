# web-playground

Minimal browser playground for the headless runtime.

## Notes

- The runtime boundary is `InputCommand -> EditorSnapshot`.
- Generate the browser package into `public/wasm`, for example with a command like `wasm-pack build crates/wasm --target web --out-dir ../../apps/web-playground/public/wasm`.
- Canvas owns rendering; the browser does not use `contenteditable` as source of truth.
