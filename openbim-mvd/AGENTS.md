# Crate instructions

`openbim-mvd` is the canonical mvdXML implementation.

- `model.rs` mirrors every mvdXML 1.1 schema type; preserve schema names in serde renames while using idiomatic Rust field names.
- `codec.rs` owns secure bounded XML parsing and canonical serialization.
- `rules.rs` owns the TemplateRule parameter language independent of IFC traversal.
- `validation.rs` owns fixed-schema and document graph/business checks.
- Add regression tests for every parser/validator defect. Do not weaken strict parsing to accept malformed input silently.
- Public APIs require rustdoc. No unsafe code.
