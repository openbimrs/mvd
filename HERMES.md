# OpenBIM.rs MVD

Canonical repository: <https://github.com/openbimrs/mvd>
Integration repository: <https://github.com/openbimrs/openbim>

Read `AGENTS.md` before changing the repository and the nested `AGENTS.md` before editing the crate. Keep the crate independently buildable; OpenBIM.rs pins it as a submodule.

## Verification

Run `./scripts/gate.sh`. It is the authoritative local and CI gate and decides success from command exit codes.

## Conventions

- Rust 2024, MSRV 1.85, AGPL-3.0-or-later.
- Pure Rust and `#![forbid(unsafe_code)]`.
- mvdXML 1.1 fixed-format validation must be distinguished from generic XSD-engine support.
- MVD does not belong in IFC. Keep IFC adapters above both families.
- Original standards PDFs/XSDs remain local under ignored `references/`; publish implementation, provenance hashes, and independently authored fixtures only.
- Use Keep a Changelog and document implemented, adapter-required, and out-of-scope capabilities honestly.
