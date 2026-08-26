#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/openbim-mvd-target}"
TEMP="$(mktemp -d)"
trap 'rm -rf "$TEMP"' EXIT

step() { printf '\n==> %s\n' "$*"; "$@"; }

step cargo fmt --all -- --check
step cargo check --workspace --all-targets
step cargo build --workspace --all-targets
step cargo test --workspace --all-targets
step cargo clippy --workspace --all-targets -- -D warnings
step env RUSTDOCFLAGS=-Dwarnings cargo doc -p openbim-mvd --lib --no-deps
step python3 scripts/check-leakage.py
step scripts/mutation-probe.sh

PACKAGE_ROOT="$ROOT"
if command -v git >/dev/null 2>&1 && ! git rev-parse --verify HEAD >/dev/null 2>&1; then
  PACKAGE_ROOT="$TEMP/source-export"
  mkdir -p "$PACKAGE_ROOT"
  cp Cargo.toml Cargo.lock LICENSE README.md "$PACKAGE_ROOT/"
  cp -a openbim-mvd "$PACKAGE_ROOT/"
fi
step cargo package --manifest-path "$PACKAGE_ROOT/openbim-mvd/Cargo.toml" --allow-dirty
PACKAGE_ARCHIVE="$CARGO_TARGET_DIR/package/openbim-mvd-0.1.0.crate"
step python3 scripts/check-leakage.py "$PACKAGE_ARCHIVE"

if command -v npm >/dev/null 2>&1; then
  step npm ci
  step npm run docs:build
  step python3 scripts/check-leakage.py docs/.vitepress/dist
else
  echo "gate: npm unavailable; documentation build NOT RUN" >&2
  exit 1
fi

if command -v git >/dev/null 2>&1 && git rev-parse --verify HEAD >/dev/null 2>&1; then
  step git diff --check
else
  echo
  echo "==> git diff --check (skipped: source tree has no commit)"
fi

echo
echo "gate: PASS"
