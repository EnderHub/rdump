#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

mkdir -p docs/generated

cargo run -p rdump --example generate_docs -- predicate-catalog \
  > docs/generated/predicate-catalog.json

cargo run -p rdump --example generate_docs -- language-matrix \
  > docs/generated/language-matrix.json

cargo run -p rdump --example generate_docs -- language-profiles \
  > docs/generated/language-profile-reference.md

cargo run -p rdump --example generate_docs -- support-matrix \
  > docs/generated/test-support-matrix.md
