//! Parser and evaluator for the mvdXML 1.1 rule grammar.

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use regex::Regex;
use thiserror::Error;

use crate::model::{LogicalOperator, TemplateRuleNode, TemplateRules};

/// A parsed mvdXML parameter or constraint expression.
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterExpression {
    pub first: BooleanTerm,
    pub rest: Vec<(LogicalConnective, BooleanTerm)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BooleanTerm {
    Comparison(Comparison),
    Group(Box<ParameterExpression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Comparison {
    pub left: Operand,
    pub operator: ComparisonOperator,
    pub right: Benchmark,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Operand {
    pub parameter: Option<String>,
    pub metric: Option<Metric>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Benchmark {
    Literal(ParameterValue),
    Regex(String),
    Parameter(Operand),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Metric {
    Value,
    Size,
    Type,
    Unique,
    Exists,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalConnective {
    And,
    Or,
    Xor,
    Nand,
    Nor,
    Nxor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    Logical(LogicalValue),
    Number(f64),
    String(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalValue {
    False,
    True,
    Unknown,
}

/// Values exposed by an IFC adapter or another rule-data source.
#[derive(Debug, Clone, Default)]
pub struct RuleValues {
    values: HashMap<(Option<String>, Option<Metric>), ParameterValue>,
}

impl RuleValues {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        parameter: impl Into<String>,
        metric: Option<Metric>,
        value: ParameterValue,
    ) -> Option<ParameterValue> {
        self.values.insert((Some(parameter.into()), metric), value)
    }

    pub fn insert_current(
        &mut self,
        metric: Metric,
        value: ParameterValue,
    ) -> Option<ParameterValue> {
        self.values.insert((None, Some(metric)), value)
    }

    #[must_use]
    pub fn get(&self, operand: &Operand) -> Option<&ParameterValue> {
        self.values
            .get(&(operand.parameter.clone(), operand.metric))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("rule expression error at byte {offset}: {message}")]
pub struct RuleParseError {
    pub offset: usize,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RuleEvaluationError {
    #[error("no value was supplied for {0}")]
    MissingOperand(OperandDisplay),
    #[error("operator {operator} is not valid for {left} and {right}")]
    Incomparable {
        operator: &'static str,
        left: &'static str,
        right: &'static str,
    },
    #[error("invalid regular expression `{pattern}`: {message}")]
    InvalidRegex { pattern: String, message: String },
}

#[derive(Debug, Error)]
pub enum TemplateRuleEvaluationError {
    #[error("template rule {index} is invalid: {source}")]
    Parse {
        index: usize,
        source: RuleParseError,
    },
    #[error("template rule {index} could not be evaluated: {source}")]
    Evaluation {
        index: usize,
        source: RuleEvaluationError,
    },
}

impl TemplateRules {
    /// Parses and evaluates this complete logical rule tree.
    pub fn evaluate(&self, values: &RuleValues) -> Result<bool, TemplateRuleEvaluationError> {
        let mut outcomes = Vec::with_capacity(self.children.len());
        for (index, child) in self.children.iter().enumerate() {
            let outcome = match child {
                TemplateRuleNode::Rule(rule) => ParameterExpression::parse(&rule.parameters)
                    .map_err(|source| TemplateRuleEvaluationError::Parse { index, source })?
                    .evaluate(values)
                    .map_err(|source| TemplateRuleEvaluationError::Evaluation { index, source })?,
                TemplateRuleNode::Group(group) => group.evaluate(values)?,
            };
            outcomes.push(outcome);
        }
        Ok(self
            .operator
            .unwrap_or(LogicalOperator::And)
            .apply(&outcomes))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperandDisplay(String);

impl fmt::Display for OperandDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl From<&Operand> for OperandDisplay {
    fn from(value: &Operand) -> Self {
        let mut output = value.parameter.clone().unwrap_or_default();
        if let Some(metric) = value.metric {
            output.push('[');
            output.push_str(metric.name());
            output.push(']');
        }
        Self(output)
    }
}

impl Metric {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Value => "Value",
            Self::Size => "Size",
            Self::Type => "Type",
            Self::Unique => "Unique",
            Self::Exists => "Exists",
        }
    }
}

impl ComparisonOperator {
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Equal => "=",
            Self::NotEqual => "!=",
            Self::Greater => ">",
            Self::GreaterOrEqual => ">=",
            Self::Less => "<",
            Self::LessOrEqual => "<=",
        }
    }
}

impl ParameterValue {
    const fn kind(&self) -> &'static str {
        match self {
            Self::Logical(_) => "logical",
            Self::Number(_) => "number",
            Self::String(_) => "string",
        }
    }
}

impl FromStr for ParameterExpression {
    type Err = RuleParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parser = Parser::new(input);
        let expression = parser.expression()?;
        parser.skip_ws();
        if parser.is_eof() {
            Ok(expression)
        } else {
            Err(parser.error("unexpected trailing input"))
        }
    }
}

impl ParameterExpression {
    pub fn parse(input: &str) -> Result<Self, RuleParseError> {
        input.parse()
    }

    pub fn evaluate(&self, values: &RuleValues) -> Result<bool, RuleEvaluationError> {
        let mut result = self.first.evaluate(values)?;
        for (operator, term) in &self.rest {
            let right = term.evaluate(values)?;
            result = operator.apply(result, right);
        }
        Ok(result)
    }

    /// Returns every named parameter referenced on either side of a comparison.
    #[must_use]
    pub fn referenced_parameters(&self) -> Vec<&str> {
        let mut output = Vec::new();
        self.collect_parameters(&mut output);
        output
    }

    fn collect_parameters<'a>(&'a self, output: &mut Vec<&'a str>) {
        self.first.collect_parameters(output);
        for (_, term) in &self.rest {
            term.collect_parameters(output);
        }
    }
}

impl BooleanTerm {
    fn evaluate(&self, values: &RuleValues) -> Result<bool, RuleEvaluationError> {
        match self {
            Self::Comparison(comparison) => comparison.evaluate(values),
            Self::Group(expression) => expression.evaluate(values),
        }
    }

    fn collect_parameters<'a>(&'a self, output: &mut Vec<&'a str>) {
        match self {
            Self::Comparison(comparison) => {
                if let Some(parameter) = &comparison.left.parameter {
                    output.push(parameter);
                }
                if let Benchmark::Parameter(operand) = &comparison.right {
                    if let Some(parameter) = &operand.parameter {
                        output.push(parameter);
                    }
                }
            }
            Self::Group(expression) => expression.collect_parameters(output),
        }
    }
}

impl Comparison {
    fn evaluate(&self, values: &RuleValues) -> Result<bool, RuleEvaluationError> {
        let left = values
            .get(&self.left)
            .ok_or_else(|| RuleEvaluationError::MissingOperand((&self.left).into()))?;
        if let Benchmark::Regex(pattern) = &self.right {
            let ParameterValue::String(value) = left else {
                return Err(RuleEvaluationError::Incomparable {
                    operator: self.operator.symbol(),
                    left: left.kind(),
                    right: "regular expression",
                });
            };
            let regex = Regex::new(pattern).map_err(|error| RuleEvaluationError::InvalidRegex {
                pattern: pattern.clone(),
                message: error.to_string(),
            })?;
            let matched = regex.is_match(value);
            return match self.operator {
                ComparisonOperator::Equal => Ok(matched),
                ComparisonOperator::NotEqual => Ok(!matched),
                _ => Err(RuleEvaluationError::Incomparable {
                    operator: self.operator.symbol(),
                    left: "string",
                    right: "regular expression",
                }),
            };
        }

        let right = match &self.right {
            Benchmark::Literal(value) => value,
            Benchmark::Parameter(operand) => values
                .get(operand)
                .ok_or_else(|| RuleEvaluationError::MissingOperand(operand.into()))?,
            Benchmark::Regex(_) => unreachable!("handled above"),
        };
        compare_values(left, self.operator, right)
    }
}

impl LogicalConnective {
    const fn apply(self, left: bool, right: bool) -> bool {
        match self {
            Self::And => left && right,
            Self::Or => left || right,
            Self::Xor => left ^ right,
            Self::Nand => !(left && right),
            Self::Nor => !(left || right),
            Self::Nxor => !(left ^ right),
        }
    }
}

fn compare_values(
    left: &ParameterValue,
    operator: ComparisonOperator,
    right: &ParameterValue,
) -> Result<bool, RuleEvaluationError> {
    let ordering = match (left, right) {
        (ParameterValue::Number(left), ParameterValue::Number(right)) => left.partial_cmp(right),
        (ParameterValue::String(left), ParameterValue::String(right)) => Some(left.cmp(right)),
        (ParameterValue::Logical(left), ParameterValue::Logical(right)) => {
            Some(logical_rank(*left).cmp(&logical_rank(*right)))
        }
        _ => None,
    };
    let Some(ordering) = ordering else {
        return Err(RuleEvaluationError::Incomparable {
            operator: operator.symbol(),
            left: left.kind(),
            right: right.kind(),
        });
    };
    Ok(match operator {
        ComparisonOperator::Equal => ordering.is_eq(),
        ComparisonOperator::NotEqual => !ordering.is_eq(),
        ComparisonOperator::Greater => ordering.is_gt(),
        ComparisonOperator::GreaterOrEqual => ordering.is_ge(),
        ComparisonOperator::Less => ordering.is_lt(),
        ComparisonOperator::LessOrEqual => ordering.is_le(),
    })
}

const fn logical_rank(value: LogicalValue) -> u8 {
    match value {
        LogicalValue::False => 0,
        LogicalValue::Unknown => 1,
        LogicalValue::True => 2,
    }
}

struct Parser<'a> {
    input: &'a str,
    offset: usize,
    depth: usize,
}

impl<'a> Parser<'a> {
    const fn new(input: &'a str) -> Self {
        Self {
            input,
            offset: 0,
            depth: 0,
        }
    }

    fn expression(&mut self) -> Result<ParameterExpression, RuleParseError> {
        self.skip_ws();
        let first = self.term()?;
        let mut rest = Vec::new();
        loop {
            self.skip_ws();
            if self.is_eof() || self.peek_char() == Some(')') {
                break;
            }
            let operator = self.logical_connective()?;
            let term = self.term()?;
            rest.push((operator, term));
        }
        Ok(ParameterExpression { first, rest })
    }

    fn term(&mut self) -> Result<BooleanTerm, RuleParseError> {
        self.skip_ws();
        if self.consume_char('(') {
            if self.depth >= 64 {
                return Err(self.error("expression nesting exceeds 64 groups"));
            }
            self.depth += 1;
            let expression = self.expression();
            self.depth -= 1;
            let expression = expression?;
            self.skip_ws();
            if !self.consume_char(')') {
                return Err(self.error("expected `)`"));
            }
            return Ok(BooleanTerm::Group(Box::new(expression)));
        }
        let left = self.operand()?;
        self.skip_ws();
        let operator = self.comparison_operator()?;
        self.skip_ws();
        let right = self.benchmark()?;
        Ok(BooleanTerm::Comparison(Comparison {
            left,
            operator,
            right,
        }))
    }

    fn operand(&mut self) -> Result<Operand, RuleParseError> {
        self.skip_ws();
        if self.peek_char() == Some('[') {
            return Ok(Operand {
                parameter: None,
                metric: Some(self.metric()?),
            });
        }
        let parameter = self.identifier()?;
        let metric = if self.peek_char() == Some('[') {
            Some(self.metric()?)
        } else {
            None
        };
        Ok(Operand {
            parameter: Some(parameter),
            metric,
        })
    }

    fn benchmark(&mut self) -> Result<Benchmark, RuleParseError> {
        self.skip_ws();
        if self.starts_keyword("reg") {
            self.offset += 3;
            self.skip_ws();
            return Ok(Benchmark::Regex(self.string_literal()?));
        }
        if self.peek_char() == Some('\'') {
            return Ok(Benchmark::Literal(ParameterValue::String(
                self.string_literal()?,
            )));
        }
        if self.peek_char().is_some_and(|value| {
            value.is_ascii_digit() || value == '+' || value == '-' || value == '.'
        }) {
            return Ok(Benchmark::Literal(ParameterValue::Number(self.number()?)));
        }
        let identifier = self.identifier()?;
        if identifier.eq_ignore_ascii_case("true") {
            return Ok(Benchmark::Literal(ParameterValue::Logical(
                LogicalValue::True,
            )));
        }
        if identifier.eq_ignore_ascii_case("false") {
            return Ok(Benchmark::Literal(ParameterValue::Logical(
                LogicalValue::False,
            )));
        }
        if identifier.eq_ignore_ascii_case("unknown") {
            return Ok(Benchmark::Literal(ParameterValue::Logical(
                LogicalValue::Unknown,
            )));
        }
        let metric = if self.peek_char() == Some('[') {
            Some(self.metric()?)
        } else {
            None
        };
        Ok(Benchmark::Parameter(Operand {
            parameter: Some(identifier),
            metric,
        }))
    }

    fn metric(&mut self) -> Result<Metric, RuleParseError> {
        if !self.consume_char('[') {
            return Err(self.error("expected metric"));
        }
        let name = self.identifier()?;
        if !self.consume_char(']') {
            return Err(self.error("expected `]` after metric"));
        }
        match name.as_str() {
            "Value" => Ok(Metric::Value),
            "Size" => Ok(Metric::Size),
            "Type" => Ok(Metric::Type),
            "Unique" => Ok(Metric::Unique),
            "Exists" => Ok(Metric::Exists),
            _ => Err(self.error("unknown metric")),
        }
    }

    fn comparison_operator(&mut self) -> Result<ComparisonOperator, RuleParseError> {
        for (token, operator) in [
            (">=", ComparisonOperator::GreaterOrEqual),
            ("<=", ComparisonOperator::LessOrEqual),
            ("!=", ComparisonOperator::NotEqual),
            ("=", ComparisonOperator::Equal),
            (">", ComparisonOperator::Greater),
            ("<", ComparisonOperator::Less),
        ] {
            if self.remaining().starts_with(token) {
                self.offset += token.len();
                return Ok(operator);
            }
        }
        Err(self.error("expected comparison operator"))
    }

    fn logical_connective(&mut self) -> Result<LogicalConnective, RuleParseError> {
        if self.consume_char('&') || self.consume_char(';') {
            return Ok(LogicalConnective::And);
        }
        if self.consume_char('|') {
            return Ok(LogicalConnective::Or);
        }
        let start = self.offset;
        let identifier = self.identifier()?;
        let operator = if identifier.eq_ignore_ascii_case("and") {
            LogicalConnective::And
        } else if identifier.eq_ignore_ascii_case("or") {
            LogicalConnective::Or
        } else if identifier.eq_ignore_ascii_case("xor") {
            LogicalConnective::Xor
        } else if identifier.eq_ignore_ascii_case("nand") {
            LogicalConnective::Nand
        } else if identifier.eq_ignore_ascii_case("nor") {
            LogicalConnective::Nor
        } else if identifier.eq_ignore_ascii_case("nxor") {
            LogicalConnective::Nxor
        } else {
            self.offset = start;
            return Err(self.error("expected logical connective"));
        };
        Ok(operator)
    }

    fn number(&mut self) -> Result<f64, RuleParseError> {
        let start = self.offset;
        if matches!(self.peek_char(), Some('+') | Some('-')) {
            self.advance_char();
        }
        let before = self.consume_digits();
        if self.consume_char('.') {
            self.consume_digits();
        } else if before == 0 {
            return Err(self.error("expected number"));
        }
        if matches!(self.peek_char(), Some('e') | Some('E')) {
            self.advance_char();
            if matches!(self.peek_char(), Some('+') | Some('-')) {
                self.advance_char();
            }
            if self.consume_digits() == 0 {
                return Err(self.error("expected exponent digits"));
            }
        }
        self.input[start..self.offset]
            .parse()
            .map_err(|_| self.error("invalid real literal"))
    }

    fn string_literal(&mut self) -> Result<String, RuleParseError> {
        if !self.consume_char('\'') {
            return Err(self.error("expected single-quoted string"));
        }
        let mut output = String::new();
        loop {
            let Some(value) = self.advance_char() else {
                return Err(self.error("unterminated string literal"));
            };
            match value {
                '\'' => return Ok(output),
                '\\' => output.push(self.escape_sequence()?),
                other => output.push(other),
            }
        }
    }

    fn escape_sequence(&mut self) -> Result<char, RuleParseError> {
        let Some(value) = self.advance_char() else {
            return Err(self.error("incomplete escape sequence"));
        };
        Ok(match value {
            'b' => '\u{0008}',
            't' => '\t',
            'n' => '\n',
            'f' => '\u{000c}',
            'r' => '\r',
            '"' => '"',
            '\'' => '\'',
            '\\' => '\\',
            'u' => {
                let start = self.offset;
                for _ in 0..4 {
                    let Some(digit) = self.advance_char() else {
                        return Err(self.error("incomplete Unicode escape"));
                    };
                    if !digit.is_ascii_hexdigit() {
                        return Err(self.error("invalid Unicode escape"));
                    }
                }
                let value = u32::from_str_radix(&self.input[start..self.offset], 16)
                    .map_err(|_| self.error("invalid Unicode escape"))?;
                char::from_u32(value).ok_or_else(|| self.error("invalid Unicode scalar"))?
            }
            digit @ '0'..='7' => {
                let mut octal = String::from(digit);
                let max_extra = if digit <= '3' { 2 } else { 1 };
                for _ in 0..max_extra {
                    if self
                        .peek_char()
                        .is_some_and(|next| matches!(next, '0'..='7'))
                    {
                        octal.push(self.advance_char().expect("peeked character"));
                    } else {
                        break;
                    }
                }
                let value = u32::from_str_radix(&octal, 8)
                    .map_err(|_| self.error("invalid octal escape"))?;
                char::from_u32(value).ok_or_else(|| self.error("invalid octal scalar"))?
            }
            _ => return Err(self.error("unsupported escape sequence")),
        })
    }

    fn identifier(&mut self) -> Result<String, RuleParseError> {
        self.skip_ws();
        let start = self.offset;
        let Some(first) = self.peek_char() else {
            return Err(self.error("expected identifier"));
        };
        if !(first.is_ascii_alphabetic() || first == '_') {
            return Err(self.error("identifier must start with an ASCII letter or underscore"));
        }
        self.advance_char();
        while self
            .peek_char()
            .is_some_and(|value| value.is_ascii_alphanumeric() || value == '_')
        {
            self.advance_char();
        }
        Ok(self.input[start..self.offset].to_owned())
    }

    fn starts_keyword(&self, keyword: &str) -> bool {
        self.remaining()
            .get(..keyword.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(keyword))
            && self
                .remaining()
                .get(keyword.len()..)
                .and_then(|rest| rest.chars().next())
                .is_some_and(|next| next == '\'' || next.is_whitespace())
    }

    fn consume_digits(&mut self) -> usize {
        let start = self.offset;
        while self.peek_char().is_some_and(|value| value.is_ascii_digit()) {
            self.advance_char();
        }
        self.offset - start
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.advance_char();
            true
        } else {
            false
        }
    }

    fn skip_ws(&mut self) {
        while self.peek_char().is_some_and(char::is_whitespace) {
            self.advance_char();
        }
    }

    fn advance_char(&mut self) -> Option<char> {
        let value = self.peek_char()?;
        self.offset += value.len_utf8();
        Some(value)
    }

    fn peek_char(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn remaining(&self) -> &'a str {
        &self.input[self.offset..]
    }

    fn is_eof(&self) -> bool {
        self.offset == self.input.len()
    }

    fn error(&self, message: impl Into<String>) -> RuleParseError {
        RuleParseError {
            offset: self.offset,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_documentation_example() {
        let expression = ParameterExpression::parse(
            "Name[Value]='HeatingInlet' AND Flow[Value]='SINK' AND Type[Value]='NOTDEFINED'",
        )
        .unwrap();
        assert_eq!(expression.referenced_parameters(), ["Name", "Flow", "Type"]);
    }

    #[test]
    fn evaluates_every_connective_and_parameter_benchmark() {
        let expression = ParameterExpression::parse(
            "(Size[Value] >= Minimum[Value] AND Enabled[Exists] = true) XOR Name[Value] = reg'^X-'",
        )
        .unwrap();
        let mut values = RuleValues::new();
        values.insert("Size", Some(Metric::Value), ParameterValue::Number(3.0));
        values.insert("Minimum", Some(Metric::Value), ParameterValue::Number(2.0));
        values.insert(
            "Enabled",
            Some(Metric::Exists),
            ParameterValue::Logical(LogicalValue::True),
        );
        values.insert(
            "Name",
            Some(Metric::Value),
            ParameterValue::String("ordinary".into()),
        );
        assert!(expression.evaluate(&values).unwrap());
    }

    #[test]
    fn rejects_excessive_group_nesting() {
        let expression = format!("{}A=1{}", "(".repeat(65), ")".repeat(65));
        let error = ParameterExpression::parse(&expression).unwrap_err();
        assert_eq!(error.message, "expression nesting exceeds 64 groups");
    }

    #[test]
    fn supports_grammar_escapes_and_semicolon_and() {
        let expression =
            ParameterExpression::parse("Name='line\\n\\u03b1';Count[Size]=2e1").unwrap();
        let BooleanTerm::Comparison(first) = expression.first else {
            panic!("expected comparison");
        };
        assert_eq!(
            first.right,
            Benchmark::Literal(ParameterValue::String("line\nα".into()))
        );
        assert_eq!(expression.rest.len(), 1);
    }

    #[test]
    fn rejects_incomplete_or_unknown_grammar() {
        for invalid in [
            "",
            "1=1",
            "Name[Other]=1",
            "Name='unterminated",
            "Name=1 AND",
        ] {
            assert!(ParameterExpression::parse(invalid).is_err(), "{invalid}");
        }
    }
}
