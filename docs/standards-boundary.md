# Standards-material boundary

The repository implements a public interchange format without redistributing locally obtained normative or official payloads. Public context is available from buildingSMART's [Model View Definition overview](https://technical.buildingsmart.org/standards/ifc/mvd/) and [Model View Definition repository](https://github.com/buildingSMART/Model-View-Definition); neither source is vendored here.

## Not included

- official mvdXML schemas;
- buildingSMART or other standards PDFs;
- official documentation examples or conformance corpora;
- copied normative prose.

Lawfully obtained local material belongs under ignored `references/`. The designated official local example is also ignored. Its integration test exits successfully when the file is absent, so clean clones, Cargo packages, and CI do not depend on it.

## Included test data

`openbim-mvd/tests/fixtures/authored-complete.mvdxml` is independently authored, non-normative test data. It exercises the typed model, writer, evaluator, and validator and is intentionally included in Cargo packages.

## Enforcement

`scripts/check-leakage.py` checks eligible source files, archive members, and the built Pages directory by both path and payload signature. The ordinary CI gate and Pages workflow run it; `scripts/mutation-probe.sh` proves a renamed schema-like payload is rejected.

The absence of a bundled schema is also a capability boundary: `validate` is the crate's fixed mvdXML 1.1 validator, not an independent formal XSD-validation result.
