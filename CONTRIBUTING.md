# Contributing

Thank you for improving `openbimrs/mvd`.

## Setup

Install Rust 1.99.1 with `rustfmt` and Clippy, Node 22+, npm, and Python 3.12+ (used by repository checks). Keep build output outside the checkout when practical:

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

## Releasing

Releases publish from CI through crates.io trusted publishing (`.github/workflows/release.yml`); no one needs a crates.io token. Bump the version, date the changelog section, run the gate, merge to `main`, then push an annotated tag `vx.y.z` on that commit. See [PUBLISHING.md](PUBLISHING.md) for the full checklist, the standards-material boundary, and how the first version of a new crate is published.

## Licensing contributions

OpenBIM.rs is dual-licensed: `AGPL-3.0-or-later OR LicenseRef-Commercial`
(see [`LICENSING.md`](LICENSING.md)). To keep the commercial option
possible, every contribution is accepted on the following terms, which you
agree to by submitting it (a pull request, patch, issue attachment or any
other material intended for inclusion):

1. **Licence to the project.** Your contribution is licensed under
   `AGPL-3.0-or-later`.
2. **Licence to the copyright holder.** You also grant the copyright holder
   named in [`NOTICE`](NOTICE) a perpetual, worldwide, non-exclusive,
   royalty-free, irrevocable licence to use, reproduce, modify, distribute,
   sublicense and relicense your contribution, including under proprietary
   or commercial terms, as part of OpenBIM.rs or works derived from it.
3. **You keep your copyright.** These terms grant a licence; they do not
   transfer ownership.
4. **You have the right to grant this.** The contribution is your own work,
   or you have permission to submit it on these terms (for example from
   your employer). Identify third-party material, keep its licence,
   attribution and provenance, and do not submit third-party material whose
   licence does not allow these terms.
5. **Patents.** You grant the same parties a licence under any patent claims
   you can license that your contribution necessarily infringes, on the terms
   of section 11 of the AGPL.

If you cannot agree to these terms, say so in the issue or pull request
before submitting; the contribution can then not be merged.
