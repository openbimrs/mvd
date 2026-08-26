# Getting started

## Requirements

- Rust 1.85 or newer
- Cargo
- No native XML library, schema download, or network service

The crate is pre-release. In this repository, use a workspace path dependency:

```toml
[dependencies]
openbim-mvd = { path = "openbim-mvd" }
```

## Parse, validate, and write

```rust
use openbim_mvd::{MvdXml, Severity};

let source = std::fs::read_to_string("view.mvdxml")?;
let document = MvdXml::from_xml(&source)?;

for issue in document.validate() {
    eprintln!("{:?} {} {}: {}", issue.severity, issue.code, issue.path, issue.message);
}

if document
    .validate()
    .iter()
    .all(|issue| issue.severity != Severity::Error)
{
    std::fs::write("canonical.mvdxml", document.to_xml()?)?;
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

`from_xml` uses conservative defaults. Use explicit limits for tighter application budgets:

```rust
use openbim_mvd::{MvdXml, ParseLimits};

let limits = ParseLimits {
    max_input_bytes: 8 * 1024 * 1024,
    max_depth: 128,
    max_events: 250_000,
    max_text_bytes: 4 * 1024 * 1024,
};
let document = MvdXml::from_xml_with_limits(source, limits)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Round-trip contract

`to_xml` is deterministic for the typed model and its output reparses to an equal `MvdXml`. This is a semantic/canonical contract, not lexical losslessness: source whitespace, comments, namespace-prefix choices, and unknown extensions are not promised to survive.

Next: [rules and validation](./rules-and-validation).
