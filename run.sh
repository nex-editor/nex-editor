#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"

case "${1:-}" in
  test)
    cargo test
    ;;
  web)
    cd "$ROOT_DIR/apps/web-playground"
    npm install
    npm run dev
    ;;
  wasm)
    cd "$ROOT_DIR/apps/web-playground"
    npm run build:wasm
    ;;
  *)
    cat <<'EOF'
Usage:
  ./run.sh test   # run Rust tests
  ./run.sh wasm   # build browser wasm package
  ./run.sh web    # install web deps and start playground
EOF
    ;;
esac
