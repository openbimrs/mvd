//! Fixed mvdXML 1.1 schema and document-graph validation.

use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::model::*;
use crate::rules::ParameterExpression;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub code: &'static str,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationOptions {
    pub business_rules: bool,
}

impl Default for ValidationOptions {
    fn default() -> Self {
        Self {
            business_rules: true,
        }
    }
}

impl MvdXml {
    /// Validates all representable XSD 1.1-shape constraints, identity/keyref
    /// constraints, rule grammar, and mvdXML document-graph business rules.
    #[must_use]
    pub fn validate(&self) -> Vec<ValidationIssue> {
        self.validate_with(ValidationOptions::default())
    }

    #[must_use]
    pub fn validate_with(&self, options: ValidationOptions) -> Vec<ValidationIssue> {
        let mut validator = Validator::new(self, options);
        validator.run();
        validator.issues
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self
            .validate()
            .iter()
            .any(|issue| issue.severity == Severity::Error)
    }
}

struct Validator<'a> {
    document: &'a MvdXml,
    options: ValidationOptions,
    issues: Vec<ValidationIssue>,
    identities: HashMap<Uuid, String>,
    templates: HashMap<Uuid, (&'a ConceptTemplate, String)>,
    concepts: HashMap<Uuid, (&'a Concept, String)>,
    exchanges: HashMap<Uuid, (&'a ExchangeRequirement, String)>,
}

impl<'a> Validator<'a> {
    fn new(document: &'a MvdXml, options: ValidationOptions) -> Self {
        Self {
            document,
            options,
            issues: Vec::new(),
            identities: HashMap::new(),
            templates: HashMap::new(),
            concepts: HashMap::new(),
            exchanges: HashMap::new(),
        }
    }

    fn run(&mut self) {
        self.validate_identity(&self.document.identity, "/mvdXML", false);
        self.index_templates();
        self.index_views();
        self.validate_templates();
        self.validate_views();
        self.validate_reference_cycles();
    }

    fn index_templates(&mut self) {
        if let Some(templates) = &self.document.templates {
            self.require_non_empty(
                &templates.concept_templates,
                "/mvdXML/Templates",
                "xsd.min_occurs",
                "Templates must contain at least one ConceptTemplate",
            );
            for (index, template) in templates.concept_templates.iter().enumerate() {
                self.index_template(
                    template,
                    format!("/mvdXML/Templates/ConceptTemplate[{index}]"),
                );
            }
        }
    }

    fn index_template(&mut self, template: &'a ConceptTemplate, path: String) {
        self.add_identity(&template.identity, &path);
        if let Some(previous) = self
            .templates
            .insert(template.identity.uuid, (template, path.clone()))
        {
            self.error(
                "xsd.template_key",
                &path,
                format!("template UUID duplicates {}", previous.1),
            );
        }
        if let Some(sub_templates) = &template.sub_templates {
            self.require_non_empty(
                &sub_templates.concept_templates,
                &format!("{path}/SubTemplates"),
                "xsd.min_occurs",
                "SubTemplates must contain at least one ConceptTemplate",
            );
            for (index, child) in sub_templates.concept_templates.iter().enumerate() {
                self.index_template(
                    child,
                    format!("{path}/SubTemplates/ConceptTemplate[{index}]"),
                );
            }
        }
    }

    fn index_views(&mut self) {
        if let Some(views) = &self.document.views {
            self.require_non_empty(
                &views.model_views,
                "/mvdXML/Views",
                "xsd.min_occurs",
                "Views must contain at least one ModelView",
            );
            for (view_index, view) in views.model_views.iter().enumerate() {
                let view_path = format!("/mvdXML/Views/ModelView[{view_index}]");
                self.add_identity(&view.identity, &view_path);
                if let Some(exchanges) = &view.exchange_requirements {
                    self.require_non_empty(
                        &exchanges.exchange_requirements,
                        &format!("{view_path}/ExchangeRequirements"),
                        "xsd.min_occurs",
                        "ExchangeRequirements must contain at least one ExchangeRequirement",
                    );
                    for (index, exchange) in exchanges.exchange_requirements.iter().enumerate() {
                        let path = format!(
                            "{view_path}/ExchangeRequirements/ExchangeRequirement[{index}]"
                        );
                        self.add_identity(&exchange.identity, &path);
                        if let Some(previous) = self
                            .exchanges
                            .insert(exchange.identity.uuid, (exchange, path.clone()))
                        {
                            self.error(
                                "xsd.exchange_key",
                                &path,
                                format!("exchange-requirement UUID duplicates {}", previous.1),
                            );
                        }
                    }
                }
                if let Some(roots) = &view.roots {
                    self.require_non_empty(
                        &roots.concept_roots,
                        &format!("{view_path}/Roots"),
                        "xsd.min_occurs",
                        "Roots must contain at least one ConceptRoot",
                    );
                    for (root_index, root) in roots.concept_roots.iter().enumerate() {
                        let root_path = format!("{view_path}/Roots/ConceptRoot[{root_index}]");
                        self.add_identity(&root.identity, &root_path);
                        if let Some(concepts) = &root.concepts {
                            self.require_non_empty(
                                &concepts.concepts,
                                &format!("{root_path}/Concepts"),
                                "xsd.min_occurs",
                                "Concepts must contain at least one Concept",
                            );
                            for (concept_index, concept) in concepts.concepts.iter().enumerate() {
                                let path = format!("{root_path}/Concepts/Concept[{concept_index}]");
                                self.add_identity(&concept.identity, &path);
                                if let Some(previous) = self
                                    .concepts
                                    .insert(concept.identity.uuid, (concept, path.clone()))
                                {
                                    self.error(
                                        "xsd.concept_key",
                                        &path,
                                        format!("concept UUID duplicates {}", previous.1),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn add_identity(&mut self, identity: &Identity, path: &str) {
        self.validate_identity(identity, path, true);
        if let Some(previous) = self.identities.insert(identity.uuid, path.to_owned()) {
            self.error(
                "xsd.unique_uuids",
                path,
                format!("UUID {} is already used at {previous}", identity.uuid),
            );
        }
    }

    fn validate_identity(&mut self, identity: &Identity, path: &str, _keyed: bool) {
        self.normalized_required(&identity.name, &format!("{path}/@name"));
        for (name, value) in [
            ("code", identity.code.as_deref()),
            ("version", identity.version.as_deref()),
            ("author", identity.author.as_deref()),
            ("owner", identity.owner.as_deref()),
            ("copyright", identity.copyright.as_deref()),
        ] {
            if let Some(value) = value {
                self.normalized(value, &format!("{path}/@{name}"));
            }
        }
    }

    fn validate_templates(&mut self) {
        let templates: Vec<_> = self.templates.values().cloned().collect();
        for (template, path) in templates {
            self.validate_definitions(
                template.definitions.as_ref(),
                &format!("{path}/Definitions"),
            );
            if template.applicable_schema.is_empty() && self.options.business_rules {
                self.warning(
                    "mvd.empty_schema_list",
                    &format!("{path}/@applicableSchema"),
                    "an empty applicableSchema list cannot match a model schema",
                );
            }
            for (index, schema) in template.applicable_schema.0.iter().enumerate() {
                self.normalized_required(schema, &format!("{path}/@applicableSchema[{index}]"));
            }
            if let Some(entities) = &template.applicable_entity {
                for (index, entity) in entities.0.iter().enumerate() {
                    self.normalized_required(entity, &format!("{path}/@applicableEntity[{index}]"));
                }
            }
            if let Some(rules) = &template.rules {
                let mut rule_ids = HashSet::new();
                self.validate_rules(rules, &format!("{path}/Rules"), &mut rule_ids);
            }
        }
    }

    fn validate_rules(&mut self, rules: &Rules, path: &str, rule_ids: &mut HashSet<String>) {
        self.require_non_empty(
            &rules.attribute_rules,
            path,
            "xsd.min_occurs",
            "Rules must contain at least one AttributeRule",
        );
        for (index, rule) in rules.attribute_rules.iter().enumerate() {
            let child_path = format!("{path}/AttributeRule[{index}]");
            self.normalized_required(
                &rule.attribute_name,
                &format!("{child_path}/@AttributeName"),
            );
            self.validate_rule_id(rule.rule_id.as_deref(), &child_path, rule_ids);
            if let Some(description) = &rule.description {
                self.normalized(description, &format!("{child_path}/@Description"));
            }
            if let Some(entity_rules) = &rule.entity_rules {
                self.validate_entity_rules(
                    entity_rules,
                    &format!("{child_path}/EntityRules"),
                    rule_ids,
                );
            }
            self.validate_constraints(
                rule.constraints.as_ref(),
                &format!("{child_path}/Constraints"),
            );
        }
    }

    fn validate_entity_rules(
        &mut self,
        rules: &EntityRules,
        path: &str,
        rule_ids: &mut HashSet<String>,
    ) {
        self.require_non_empty(
            &rules.entity_rules,
            path,
            "xsd.min_occurs",
            "EntityRules must contain at least one EntityRule",
        );
        for (index, rule) in rules.entity_rules.iter().enumerate() {
            let child_path = format!("{path}/EntityRule[{index}]");
            self.normalized_required(&rule.entity_name, &format!("{child_path}/@EntityName"));
            self.validate_rule_id(rule.rule_id.as_deref(), &child_path, rule_ids);
            if let Some(description) = &rule.description {
                self.normalized(description, &format!("{child_path}/@Description"));
            }
            if let Some(reference) = &rule.references {
                self.validate_reference(
                    &reference.template,
                    &format!("{child_path}/References/Template"),
                    ReferenceKind::Template,
                );
                if let Some(prefix) = &reference.id_prefix {
                    self.validate_rule_id(
                        Some(prefix),
                        &format!("{child_path}/References"),
                        &mut HashSet::new(),
                    );
                }
            }
            if let Some(attribute_rules) = &rule.attribute_rules {
                self.validate_rules(
                    attribute_rules,
                    &format!("{child_path}/AttributeRules"),
                    rule_ids,
                );
            }
            self.validate_constraints(
                rule.constraints.as_ref(),
                &format!("{child_path}/Constraints"),
            );
        }
    }

    fn validate_constraints(&mut self, constraints: Option<&Constraints>, path: &str) {
        let Some(constraints) = constraints else {
            return;
        };
        self.require_non_empty(
            &constraints.constraints,
            path,
            "xsd.min_occurs",
            "Constraints must contain at least one Constraint",
        );
        for (index, constraint) in constraints.constraints.iter().enumerate() {
            let expression_path = format!("{path}/Constraint[{index}]/@Expression");
            self.normalized(&constraint.expression, &expression_path);
            if let Err(error) = ParameterExpression::parse(&constraint.expression) {
                self.error("mvd.rule_grammar", &expression_path, error.to_string());
            }
        }
    }

    fn validate_rule_id(
        &mut self,
        id: Option<&str>,
        path: &str,
        sibling_ids: &mut HashSet<String>,
    ) {
        let Some(id) = id else { return };
        let valid = id
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
            && id
                .chars()
                .all(|value| value.is_ascii_alphanumeric() || value == '_');
        if !valid {
            self.error(
                "xsd.rule_id",
                &format!("{path}/@RuleID"),
                "RuleID must match [a-zA-Z_][a-zA-Z0-9_]*",
            );
        }
        if !sibling_ids.insert(id.to_owned()) {
            self.error(
                "mvd.ambiguous_rule_id",
                &format!("{path}/@RuleID"),
                format!("RuleID `{id}` is duplicated in the same rule scope"),
            );
        }
    }

    fn validate_views(&mut self) {
        let Some(views) = &self.document.views else {
            return;
        };
        for (view_index, view) in views.model_views.iter().enumerate() {
            let view_path = format!("/mvdXML/Views/ModelView[{view_index}]");
            self.normalized(
                &view.applicable_schema,
                &format!("{view_path}/@applicableSchema"),
            );
            if view.applicable_schema.is_empty() && self.options.business_rules {
                self.warning(
                    "mvd.empty_schema",
                    &format!("{view_path}/@applicableSchema"),
                    "an empty applicableSchema cannot select an IFC schema",
                );
            }
            self.validate_definitions(
                view.definitions.as_ref(),
                &format!("{view_path}/Definitions"),
            );
            if let Some(base_view) = &view.base_view {
                self.validate_reference(
                    base_view,
                    &format!("{view_path}/BaseView"),
                    ReferenceKind::Generic,
                );
            }
            if let Some(exchanges) = &view.exchange_requirements {
                for (index, exchange) in exchanges.exchange_requirements.iter().enumerate() {
                    self.validate_definitions(
                        exchange.definitions.as_ref(),
                        &format!("{view_path}/ExchangeRequirements/ExchangeRequirement[{index}]/Definitions"),
                    );
                }
            }
            if let Some(roots) = &view.roots {
                for (root_index, root) in roots.concept_roots.iter().enumerate() {
                    self.validate_root(
                        root,
                        &format!("{view_path}/Roots/ConceptRoot[{root_index}]"),
                        view,
                    );
                }
            }
        }
    }

    fn validate_root(&mut self, root: &ConceptRoot, path: &str, view: &ModelView) {
        self.validate_definitions(root.definitions.as_ref(), &format!("{path}/Definitions"));
        if let Some(entity) = &root.applicable_root_entity {
            self.normalized(entity, &format!("{path}/@applicableRootEntity"));
        }
        if let Some(applicability) = &root.applicability {
            self.validate_definitions(
                applicability.definitions.as_ref(),
                &format!("{path}/Applicability/Definitions"),
            );
            self.validate_reference(
                &applicability.template,
                &format!("{path}/Applicability/Template"),
                ReferenceKind::Template,
            );
            self.validate_template_rules(
                &applicability.template_rules,
                &format!("{path}/Applicability/TemplateRules"),
                applicability.template.reference,
            );
            self.validate_template_schema(
                applicability.template.reference,
                view,
                &format!("{path}/Applicability/Template"),
            );
        }
        if let Some(concepts) = &root.concepts {
            for (index, concept) in concepts.concepts.iter().enumerate() {
                self.validate_concept(concept, &format!("{path}/Concepts/Concept[{index}]"), view);
            }
        }
    }

    fn validate_concept(&mut self, concept: &Concept, path: &str, view: &ModelView) {
        self.validate_definitions(concept.definitions.as_ref(), &format!("{path}/Definitions"));
        self.validate_reference(
            &concept.template,
            &format!("{path}/Template"),
            ReferenceKind::Template,
        );
        self.validate_template_rules(
            &concept.template_rules,
            &format!("{path}/TemplateRules"),
            concept.template.reference,
        );
        self.validate_template_schema(
            concept.template.reference,
            view,
            &format!("{path}/Template"),
        );
        if concept.override_base == Some(true) && concept.base_concept.is_none() {
            self.error(
                "mvd.override_without_base",
                &format!("{path}/@override"),
                "override=true requires baseConcept",
            );
        }
        if let Some(base) = concept.base_concept {
            if !self.concepts.contains_key(&base) {
                self.error(
                    "xsd.concept_keyref",
                    &format!("{path}/@baseConcept"),
                    format!("baseConcept {base} does not identify a Concept"),
                );
            }
        }
        if let Some(requirements) = &concept.requirements {
            self.require_non_empty(
                &requirements.requirements,
                &format!("{path}/Requirements"),
                "xsd.min_occurs",
                "Requirements must contain at least one Requirement",
            );
            for (index, requirement) in requirements.requirements.iter().enumerate() {
                let requirement_path = format!("{path}/Requirements/Requirement[{index}]");
                let Some(exchange) = self
                    .exchanges
                    .get(&requirement.exchange_requirement)
                    .map(|(exchange, _)| *exchange)
                else {
                    self.error(
                        "xsd.exchange_keyref",
                        &format!("{requirement_path}/@exchangeRequirement"),
                        format!(
                            "{} does not identify an ExchangeRequirement",
                            requirement.exchange_requirement
                        ),
                    );
                    continue;
                };
                if self.options.business_rules
                    && !applicability_overlaps(requirement.applicability, exchange.applicability)
                {
                    self.error(
                        "mvd.exchange_applicability",
                        &format!("{requirement_path}/@applicability"),
                        "requirement applicability does not overlap its ExchangeRequirement",
                    );
                }
            }
        }
    }

    fn validate_template_schema(&mut self, reference: Option<Uuid>, view: &ModelView, path: &str) {
        if !self.options.business_rules {
            return;
        }
        let Some(reference) = reference else { return };
        let Some(template) = self
            .templates
            .get(&reference)
            .map(|(template, _)| *template)
        else {
            return;
        };
        if !template.applicable_schema.0.is_empty()
            && !template
                .applicable_schema
                .0
                .iter()
                .any(|schema| schema == &view.applicable_schema)
        {
            self.error(
                "mvd.template_schema_mismatch",
                path,
                format!(
                    "template does not declare model-view schema `{}` in applicableSchema",
                    view.applicable_schema
                ),
            );
        }
    }

    fn validate_template_rules(
        &mut self,
        rules: &TemplateRules,
        path: &str,
        template_id: Option<Uuid>,
    ) {
        self.require_non_empty(
            &rules.children,
            path,
            "xsd.min_occurs",
            "TemplateRules must contain at least one rule or group",
        );
        if matches!(rules.operator, Some(LogicalOperator::Not)) && rules.children.len() != 1 {
            self.warning(
                "mvd.not_arity",
                &format!("{path}/@operator"),
                "NOT is interoperable only with one child; multiple children are treated as NOT(AND(...))",
            );
        }
        let available = template_id
            .and_then(|id| self.templates.get(&id).map(|(template, _)| *template))
            .map(collect_template_rule_ids);
        for (index, child) in rules.children.iter().enumerate() {
            match child {
                TemplateRuleNode::Group(group) => self.validate_template_rules(
                    group,
                    &format!("{path}/TemplateRules[{index}]"),
                    template_id,
                ),
                TemplateRuleNode::Rule(rule) => {
                    let expression_path = format!("{path}/TemplateRule[{index}]/@Parameters");
                    self.normalized(&rule.parameters, &expression_path);
                    match ParameterExpression::parse(&rule.parameters) {
                        Ok(expression) if self.options.business_rules => {
                            if let Some(available) = &available {
                                for parameter in expression.referenced_parameters() {
                                    if !available.contains(parameter) {
                                        self.error(
                                            "mvd.unknown_rule_id",
                                            &expression_path,
                                            format!("parameter `{parameter}` is not declared by the referenced template"),
                                        );
                                    }
                                }
                            }
                        }
                        Ok(_) => {}
                        Err(error) => {
                            self.error("mvd.rule_grammar", &expression_path, error.to_string())
                        }
                    }
                }
            }
        }
    }

    fn validate_reference(
        &mut self,
        reference: &GenericReference,
        path: &str,
        kind: ReferenceKind,
    ) {
        if self.options.business_rules && reference.reference.is_none() && reference.href.is_none()
        {
            self.error(
                "mvd.empty_reference",
                path,
                "a reference must provide ref, href, or both",
            );
        }
        if kind == ReferenceKind::Template {
            if let Some(id) = reference.reference {
                if !self.templates.contains_key(&id) {
                    self.error(
                        "xsd.template_keyref",
                        &format!("{path}/@ref"),
                        format!("{id} does not identify a ConceptTemplate"),
                    );
                }
            }
        }
    }

    fn validate_definitions(&mut self, definitions: Option<&Definitions>, path: &str) {
        let Some(definitions) = definitions else {
            return;
        };
        self.require_non_empty(
            &definitions.definitions,
            path,
            "xsd.min_occurs",
            "Definitions must contain at least one Definition",
        );
        for (index, definition) in definitions.definitions.iter().enumerate() {
            let definition_path = format!("{path}/Definition[{index}]");
            if let Some(body) = &definition.body {
                if let Some(lang) = &body.lang {
                    self.validate_language(lang, &format!("{definition_path}/Body/@lang"));
                }
                if let Some(tags) = &body.tags {
                    for (tag_index, tag) in tags.0.iter().enumerate() {
                        self.normalized_required(
                            tag,
                            &format!("{definition_path}/Body/@tags[{tag_index}]"),
                        );
                    }
                }
            }
            for (link_index, link) in definition.links.iter().enumerate() {
                if let Some(lang) = &link.lang {
                    self.validate_language(
                        lang,
                        &format!("{definition_path}/Link[{link_index}]/@lang"),
                    );
                }
                if let Some(title) = &link.title {
                    self.normalized(
                        title,
                        &format!("{definition_path}/Link[{link_index}]/@title"),
                    );
                }
            }
        }
    }

    fn validate_language(&mut self, lang: &str, path: &str) {
        let valid = !lang.is_empty()
            && lang.split('-').all(|part| {
                !part.is_empty()
                    && part.len() <= 8
                    && part.chars().all(|value| value.is_ascii_alphanumeric())
            });
        if !valid {
            self.error("xsd.language", path, "invalid xs:language value");
        }
    }

    fn validate_reference_cycles(&mut self) {
        for (id, (_, path)) in self.concepts.clone() {
            let mut visiting = HashSet::new();
            let mut current = Some(id);
            while let Some(next) = current {
                if !visiting.insert(next) {
                    self.error(
                        "mvd.base_concept_cycle",
                        &format!("{path}/@baseConcept"),
                        "baseConcept chain contains a cycle",
                    );
                    break;
                }
                current = self
                    .concepts
                    .get(&next)
                    .and_then(|(concept, _)| concept.base_concept);
            }
        }
        for (id, (_, path)) in self.templates.clone() {
            let mut visiting = HashSet::new();
            if self.template_cycle(id, &mut visiting, &mut HashSet::new()) {
                self.error(
                    "mvd.template_reference_cycle",
                    &path,
                    "partial-template References contain a cycle",
                );
            }
        }
    }

    fn template_cycle(
        &self,
        id: Uuid,
        visiting: &mut HashSet<Uuid>,
        visited: &mut HashSet<Uuid>,
    ) -> bool {
        if visited.contains(&id) {
            return false;
        }
        if !visiting.insert(id) {
            return true;
        }
        if let Some((template, _)) = self.templates.get(&id) {
            let mut refs = Vec::new();
            collect_template_references(template, &mut refs);
            for child in refs {
                if self.templates.contains_key(&child)
                    && self.template_cycle(child, visiting, visited)
                {
                    return true;
                }
            }
        }
        visiting.remove(&id);
        visited.insert(id);
        false
    }

    fn normalized_required(&mut self, value: &str, path: &str) {
        self.normalized(value, path);
        // `xs:normalizedString` permits an empty value.
    }

    fn normalized(&mut self, value: &str, path: &str) {
        if value.contains(['\r', '\n', '\t']) {
            self.error(
                "xsd.normalized_string",
                path,
                "normalizedString cannot contain tab or line-break characters",
            );
        }
    }

    fn require_non_empty<T>(
        &mut self,
        values: &[T],
        path: &str,
        code: &'static str,
        message: &str,
    ) {
        if values.is_empty() {
            self.error(code, path, message);
        }
    }

    fn error(&mut self, code: &'static str, path: &str, message: impl Into<String>) {
        self.issues.push(ValidationIssue {
            severity: Severity::Error,
            code,
            path: path.to_owned(),
            message: message.into(),
        });
    }

    fn warning(&mut self, code: &'static str, path: &str, message: impl Into<String>) {
        self.issues.push(ValidationIssue {
            severity: Severity::Warning,
            code,
            path: path.to_owned(),
            message: message.into(),
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReferenceKind {
    Generic,
    Template,
}

fn collect_template_rule_ids(template: &ConceptTemplate) -> HashSet<String> {
    let mut output = HashSet::new();
    if let Some(rules) = &template.rules {
        collect_rule_ids(rules, "", &mut output);
    }
    output
}

fn collect_rule_ids(rules: &Rules, prefix: &str, output: &mut HashSet<String>) {
    for rule in &rules.attribute_rules {
        if let Some(id) = &rule.rule_id {
            output.insert(format!("{prefix}{id}"));
        }
        if let Some(entity_rules) = &rule.entity_rules {
            for entity in &entity_rules.entity_rules {
                if let Some(id) = &entity.rule_id {
                    output.insert(format!("{prefix}{id}"));
                }
                if let Some(rules) = &entity.attribute_rules {
                    collect_rule_ids(rules, prefix, output);
                }
            }
        }
    }
}

fn collect_template_references(template: &ConceptTemplate, output: &mut Vec<Uuid>) {
    if let Some(rules) = &template.rules {
        collect_rule_references(rules, output);
    }
}

fn collect_rule_references(rules: &Rules, output: &mut Vec<Uuid>) {
    for attribute in &rules.attribute_rules {
        if let Some(entity_rules) = &attribute.entity_rules {
            for entity in &entity_rules.entity_rules {
                if let Some(reference) = &entity.references {
                    if let Some(id) = reference.template.reference {
                        output.push(id);
                    }
                }
                if let Some(children) = &entity.attribute_rules {
                    collect_rule_references(children, output);
                }
            }
        }
    }
}

fn applicability_overlaps(left: Option<Applicability>, right: Option<Applicability>) -> bool {
    let left = left.unwrap_or(Applicability::Both);
    let right = right.unwrap_or(Applicability::Both);
    left == Applicability::Both || right == Applicability::Both || left == right
}
