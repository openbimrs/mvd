# Capabilities

“Supports mvdXML” is too broad to be useful. This table separates implemented document behavior from adapter-owned IFC execution.

| Area | Status | Evidence and boundary |
|---|---:|---|
| mvdXML version | Implemented | Fixed mvdXML 1.1 namespace and typed vocabulary |
| Typed data model | Implemented | Public structs/enums mirror the representable 1.1 document structure |
| XML parsing | Implemented | Namespace-aware typed deserialization after bounded preflight checks |
| XML safety budgets | Implemented | Configurable input bytes, depth, events, and aggregate text/CDATA |
| DTD handling | Implemented | DTD declarations are rejected; no external resolution or network fallback |
| Schema child order | Implemented | Known parent/child sequences are checked before deserialization |
| Unknown XML preservation | Absent | Unknown fields are rejected; this is not a lossless generic XML tree |
| Deterministic writing | Implemented | XML declaration, mvdXML namespace, typed field order, semantic reparse equality |
| Lexical/byte round trips | Absent | Prefix choices, formatting, comments, and other lexical details are not promised |
| Parameter-expression parser | Implemented | Comparisons, all metrics, literals, regex benchmarks, parameter benchmarks, parentheses, and logical connectives |
| Parameter-expression evaluator | Implemented | Evaluates `RuleValues` supplied by the caller |
| Nested `TemplateRules` | Implemented | Recursive groups with effective logical operators |
| Structural validation | Implemented | Required content, cardinality, fixed values, and type-specific checks |
| Identity and graph validation | Implemented | UUID uniqueness, keys/keyrefs, template/exchange/concept references, cycles, and business rules |
| Generic XSD engine | Absent | Validation is code for the fixed 1.1 model, not arbitrary schema evaluation |
| IFC loading/traversal | Absent | No IFC parser dependency or graph adapter |
| MVD-vs-IFC execution | Absent | The crate cannot determine IFC conformance by itself |
| CLI / Python bindings | Absent | Rust library API only |
| Official standards payload | Absent by policy | No official schema, PDF, or example is bundled |

## What an IFC integration must add

An adapter owns IFC schema/entity traversal, template path interpretation, metric extraction, exchange selection, and the mapping into `RuleValues`. It can then call the rule evaluator and aggregate results. Keeping that adapter separate prevents the mvdXML document crate from claiming IFC behavior it does not implement.

## Validation is not formal XSD validation

The validator implements the fixed schema shape, cardinality, identity constraints, rule grammar, and mvdXML graph/business checks represented by this crate. It neither accepts an arbitrary schema nor bundles the official schema. Applications that require an independent formal-validation result must use a separately sourced validator and lawfully obtained schema.
