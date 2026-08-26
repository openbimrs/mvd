//! buildingSMART mvdXML 1.1 for Rust.
//!
//! This crate provides complete typed bindings for the mvdXML 1.1 schema,
//! bounded DTD-free parsing, canonical writing, template-rule parsing and
//! evaluation, and fixed-schema/document-graph validation. IFC graph traversal
//! is intentionally adapter-owned and not claimed by this crate.

#![forbid(unsafe_code)]

pub mod codec;
pub mod model;
pub mod rules;
pub mod validation;

pub use codec::{CodecError, ParseLimits};
pub use model::*;
pub use rules::{
    Comparison, Metric, ParameterExpression, ParameterValue, RuleEvaluationError, RuleParseError,
    RuleValues, TemplateRuleEvaluationError,
};
pub use validation::{Severity, ValidationIssue, ValidationOptions};
