# Contributing

Thank you for improving `openbimrs/mvd`.

## Setup

Install Rust 1.85.0 with `rustfmt` and Clippy, Node 22+, npm, and Python 3.12+ (used by repository checks). Keep build output outside the checkout when practical:

```bash
export CARGO_TARGET_DIR=/tmp/openbim-mvd-target
npm ci
./scripts/gate.sh
```

## Project boundaries

- Keep `openbim-mvd` pure Rust and focused on mvdXML document semantics.
- Do not add IFC graph traversal or claim that evaluating caller-supplied rule values executes MVD concepts against IFC.
- Preserve the one-way integration boundary: an adapter may depend on MVD and IFC libraries; neither core library should depend on that adapter.
- Do not add official PDFs, schemas, copied standards prose, or restricted examples. Put lawfully obtained local material under ignored `references/` and the designated ignored local fixture path.
- Synthetic fixtures must be independently authored and clearly non-normative.
- Keep parser limits and strict namespace/order/UUID checks intact. Add a regression test for every parser or validator defect.
- Public Rust APIs require rustdoc, and unsafe code remains forbidden.

## Required checks

```bash
./scripts/gate.sh
```

The gate checks formatting, compilation, tests, Clippy, rustdoc, mutation sensitivity, standards-payload leakage, the Cargo package, and the VitePress build. The `official_local` test is optional by design and must continue to skip when its ignored fixture is absent.

## Pull requests

Keep changes focused. Explain standards assumptions, call out capability-boundary changes, add an `[Unreleased]` changelog entry for user-visible behavior, and report the exact checks run. Never attach restricted standards files to an issue or pull request.

## Licensing contributions

Unless an explicitly signed agreement says otherwise, every contribution
submitted to this repository is licensed under `AGPL-3.0-or-later`. Submit only
work that you have the right to license. Identify third-party material and
preserve its license, attribution, and provenance.
