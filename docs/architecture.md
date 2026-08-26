# Architecture

## Layers

1. **Typed model** — `model.rs` represents the mvdXML 1.1 document, templates, views, concepts, requirements, definitions, identities, and rule groups.
2. **Codec** — `codec.rs` performs bounded namespace/order/UUID preflight checks, typed deserialization, and deterministic serialization.
3. **Rule language** — `rules.rs` parses and evaluates parameter expressions and recursive `TemplateRules` using caller-owned values.
4. **Validation** — `validation.rs` indexes identities and graph relationships, then applies fixed-schema, cardinality, key/keyref, grammar, cycle, and business checks.

## Dependency boundary

The crate depends on general Rust libraries for XML, serialization, regular expressions, errors, and UUIDs. It has no IFC dependency.

A complete IFC conformance tool belongs in a higher-level adapter:

```text
IFC library ─┐
             ├─> application adapter ─> RuleValues ─> openbim-mvd evaluator
openbim-mvd ─┘
```

The adapter interprets template paths, traverses IFC data, selects exchanges, computes metrics, and aggregates concept results. `openbim-mvd` remains responsible only for mvdXML document semantics and evaluation of supplied values.

## Invariants

- mvdXML 1.1 is the only claimed document version.
- Untrusted XML is budgeted before typed deserialization.
- Writing is deterministic but not lexical-lossless.
- Validation is fixed-format code, not a generic XSD engine.
- IFC graph traversal never hides behind document validation.
- Official standards payloads do not enter source, package, or Pages output.
