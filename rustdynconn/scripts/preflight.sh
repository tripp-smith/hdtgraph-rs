#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

cargo fmt --check
RUSTFLAGS='-D warnings' cargo test -q
python -m pytest -m "core" tests/python -q
