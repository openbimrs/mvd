# Rules and validation

## Evaluate a parameter expression

```rust
use openbim_mvd::{Metric, ParameterExpression, ParameterValue, RuleValues};

let expression = ParameterExpression::parse("_Name[Value]='Wall'")?;
let mut values = RuleValues::new();
values.insert(
    "_Name",
    Some(Metric::Value),
    ParameterValue::String("Wall".to_owned()),
);
assert!(expression.evaluate(&values)?);
# Ok::<(), Box<dyn std::error::Error>>(())
```

The grammar covers value, size, type, uniqueness, and existence metrics; string, numeric, logical, regex, and parameter benchmarks; comparison operators; parentheses; and AND/OR/XOR/NAND/NOR/NXOR-style logical connectives. Nested `TemplateRules` groups use the same evaluator recursively.

`RuleValues` is the integration seam. The crate does not know how a parameter maps to an IFC entity or attribute. Your adapter must traverse the IFC model and supply each value and metric explicitly.

## Validate a document

```rust
use openbim_mvd::{MvdXml, Severity, ValidationOptions};

let document = MvdXml::from_xml(source)?;
let structural_only = document.validate_with(ValidationOptions {
    business_rules: false,
});
let all_issues = document.validate();
let valid = !all_issues
    .iter()
    .any(|issue| issue.severity == Severity::Error);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Diagnostics carry a severity, stable code, document path, and message. Validation includes representable fixed-schema constraints, cardinality, UUID identities, keys/keyrefs, rule syntax, reference cycles, and mvdXML-specific business rules. `business_rules: false` disables the business-rule layer; it does not turn the crate into a generic XSD validator.
