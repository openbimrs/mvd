# Repository instructions

This repository implements buildingSMART mvdXML as the canonical `openbimrs/mvd` family.

## Layout

- `openbim-mvd/`: pure-Rust typed model, codec, rule language, and validation.
- `docs/`: VitePress user, architecture, security, and project documentation.
- `scripts/`: authoritative gate, mutation probes, and leakage checks.
- `tests/fixtures/`: independently authored redistributable mvdXML fixtures.
- `references/`: ignored local standards material; never package or publish it.

## Boundaries

The crate owns mvdXML document semantics, expression rules, and internal cross-reference validation. It does not parse IFC or claim to execute IFC graph paths. An integration adapter may consume both `openbim-mvd` and IFC APIs; neither core family reverses that dependency.

Run `./scripts/gate.sh` before committing. Update the capability matrix whenever behavior changes.
