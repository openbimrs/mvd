# Publishing and provenance

`openbim-mvd` is pre-release software. The Cargo manifest permits packaging, but this repository has no automated release workflow. Do not publish merely because `cargo publish` is technically available.

## Standards-material boundary

The implementation is MIT-licensed. Redistribution rights for locally obtained mvdXML schemas, PDFs, and official example documents are not assumed. They are not runtime or build dependencies and must not appear in the repository, crate archive, or documentation site.

- `references/` is ignored and reserved for local material.
- `tests/fixtures/mvdXML_V1-1-Final-Documentation.xml` is ignored and used only by an optional local test.
- `openbim-mvd/tests/fixtures/authored-complete.mvdxml` is independently authored, non-normative, and included in the crate package.
- `scripts/check-leakage.py` scans source names and bytes, archives, and the generated Pages tree.

## Release checklist

1. Confirm the intended version and update `CHANGELOG.md`.
2. Run the complete gate with Rust 1.85:

   ```bash
   export CARGO_TARGET_DIR=/tmp/openbim-mvd-target
   ./scripts/gate.sh
   ```

3. Inspect the package contents and leakage result:

   ```bash
   cargo package -p openbim-mvd --allow-dirty --list
   python3 scripts/check-leakage.py      "$CARGO_TARGET_DIR/package/openbim-mvd-0.1.0.crate"
   ```

4. Verify a clean install and API documentation against the MSRV.
5. Confirm repository ownership, crate metadata, tag, and crates.io ownership with two maintainers.
6. Publish manually only from a clean, reviewed commit; create and verify the signed/tagged GitHub release separately.
7. Confirm the package on crates.io and API documentation on docs.rs before updating release links.

A future trusted-publishing workflow requires a separate security review and protected GitHub environment. It is not part of the current repository.
