# nex-editor
nex-editor is a rich text editor based on rust, support pageing like word.

## Feature
- [ ] paragraph + text render

## Web Playground

The repository now includes a minimal WASM-driven plain text playground at `apps/web-playground`.

```bash
cd apps/web-playground
npm install
npm run dev
```

That flow builds the WASM package from `crates/wasm`, writes it into `apps/web-playground/public/wasm`, and starts Vite.
