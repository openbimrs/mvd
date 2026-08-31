# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and releases are intended to follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed

- Relicensed repository-authored work from MIT to `AGPL-3.0-or-later`; historical releases remain under their published MIT terms, and third-party material retains its own terms.

### Added

- Standalone `openbim-mvd` pure-Rust crate targeting mvdXML 1.1.
- Complete typed document model for the fixed mvdXML 1.1 vocabulary.
- Namespace-aware, schema-order-aware XML parser with configurable input, depth, event, and text limits and no DTD support.
- Deterministic namespace-qualified writer with semantic round-trip coverage.
- Parameter-expression parser and evaluator, including nested `TemplateRules` groups and caller-supplied rule values.
- Fixed-schema, cardinality, identity, key/keyref, graph, rule-grammar, and business validation.
- Independently authored publishable conformance fixture and optional ignored local-reference test.
- CI, GitHub Pages documentation, package/source leakage checks, and mutation probes.

### Security

- Excluded locally supplied standards PDFs, schemas, and official examples from source, Cargo packages, and Pages artifacts.
- Added bounded XML preflight checks and DTD rejection before typed deserialization.
- Rejects undeclared elements, non-schema text content, wrongly qualified model/XSI attributes, and content on `xsi:nil="true"` elements.
- Caps parameter-expression grouping depth to prevent recursive parser exhaustion.

### Fixed

- Kept the crate compatible with its declared Rust 1.85 MSRV by avoiding newer let-chain syntax.

[Unreleased]: https://github.com/openbimrs/mvd/commits/main
