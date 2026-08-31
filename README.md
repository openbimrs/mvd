# openbimrs/mvd

[![CI](https://github.com/openbimrs/mvd/actions/workflows/ci.yml/badge.svg)](https://github.com/openbimrs/mvd/actions/workflows/ci.yml)
[![Docs](https://github.com/openbimrs/mvd/actions/workflows/pages.yml/badge.svg)](https://openbimrs.github.io/mvd/)
[![MSRV 1.85](https://img.shields.io/badge/MSRV-1.85-blue)](rust-toolchain.toml)
[![License: AGPL-3.0-or-later](https://img.shields.io/badge/License-AGPL--3.0--or--later-blue.svg)](LICENSE)

Pure-Rust, typed tooling for buildingSMART mvdXML 1.1 documents: bounded XML parsing, deterministic writing, parameter-rule evaluation, and document validation.

**Documentation:** [start here](https://openbimrs.github.io/mvd/) · [capabilities](https://openbimrs.github.io/mvd/capabilities) · [guide](https://openbimrs.github.io/mvd/guide/getting-started) · [architecture](https://openbimrs.github.io/mvd/architecture) · [security](https://openbimrs.github.io/mvd/security) · [standards boundary](https://openbimrs.github.io/mvd/standards-boundary) · [changelog](CHANGELOG.md)

> **Pre-release:** the crate is version `0.1.0`. There is no automated release workflow or compatibility guarantee yet.

## Capability matrix

| Capability | Status | Boundary |
|---|---:|---|
| mvdXML 1.1 typed model | Yes | Models the fixed mvdXML 1.1 vocabulary with public Rust types |
| Namespace-aware parsing | Yes | Requires the mvdXML 1.1 root and namespace; checks schema child order and canonical UUID spelling |
| Bounded, DTD-free XML handling | Yes | Default byte, depth, event, and text budgets; DTD declarations are rejected |
| Deterministic writing | Yes | Emits an XML declaration and canonical mvdXML namespace; semantic round trips are tested |
| Parameter-expression parsing/evaluation | Yes | Full comparison, metric, literal, regex, grouping, and logical-connective grammar |
| Nested `TemplateRules` evaluation | Yes | Evaluates caller-supplied `RuleValues`; it does not obtain values from IFC |
| Fixed-schema and graph validation | Yes | Cardinality, identity, key/keyref, rule grammar, cycles, and mvdXML business checks |
| Generic XSD validation | No | No XSD processor or bundled schema is provided |
| IFC graph traversal or concept execution | **No** | An integration adapter must map IFC data into rule values |
| Lossless preservation of unknown XML | No | The model is strict and typed; the writer is deterministic, not byte-preserving |
| CLI or Python API | No | The public surface is the Rust library |
| Bundled official schemas, PDFs, or fixtures | **No** | Restricted/local material stays ignored; one independently authored fixture is packaged |

## Rust

The crate currently targets Rust 1.85 and edition 2024. In a checkout:

```toml
[dependencies]
openbim-mvd = { path = "openbim-mvd" }
```

```rust
use openbim_mvd::{MvdXml, Severity};

let xml = r#"<mvdXML xmlns="http://buildingsmart-tech.org/mvd/XML/1.1"
    uuid="00000000-0000-4000-8000-000000000001" name="example"/>"#;
let document = MvdXml::from_xml(xml)?;
let errors = document
    .validate()
    .into_iter()
    .filter(|issue| issue.severity == Severity::Error)
    .count();
assert_eq!(errors, 0);
let canonical_xml = document.to_xml()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Rule evaluation is deliberately independent of IFC traversal:

```rust
use openbim_mvd::{Metric, ParameterExpression, ParameterValue, RuleValues};

let expression = ParameterExpression::parse("_Name[Value]='Wall'")?;
let mut values = RuleValues::new();
values.insert(
    "_Name",
    Some(Metric::Value),
    ParameterValue::String("Wall".into()),
);
assert!(expression.evaluate(&values)?);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Development

```bash
export CARGO_TARGET_DIR=/tmp/openbim-mvd-target
./scripts/gate.sh
```

The optional local test `official_local` returns early when its ignored reference fixture is absent, so clean clones and CI never depend on restricted material.

See [CONTRIBUTING.md](CONTRIBUTING.md), [PUBLISHING.md](PUBLISHING.md), and [SECURITY.md](SECURITY.md).
