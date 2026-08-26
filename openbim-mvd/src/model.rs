//! Complete typed bindings for the mvdXML 1.1 schema.

use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

/// The mvdXML 1.1 namespace.
pub const NAMESPACE: &str = "http://buildingsmart-tech.org/mvd/XML/1.1";

/// A complete mvdXML 1.1 document.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename = "mvdXML")]
pub struct MvdXml {
    #[serde(rename = "Templates", skip_serializing_if = "Option::is_none")]
    pub templates: Option<Templates>,
    #[serde(rename = "Views", skip_serializing_if = "Option::is_none")]
    pub views: Option<Views>,
    #[serde(flatten)]
    pub identity: Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Templates {
    #[serde(rename = "ConceptTemplate")]
    pub concept_templates: Vec<ConceptTemplate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Views {
    #[serde(rename = "ModelView")]
    pub model_views: Vec<ModelView>,
}

/// Metadata shared by every independently identifiable mvdXML object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    #[serde(rename = "@uuid")]
    pub uuid: Uuid,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@code", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(rename = "@version", skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(rename = "@status", skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    #[serde(rename = "@author", skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(rename = "@owner", skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(rename = "@copyright", skip_serializing_if = "Option::is_none")]
    pub copyright: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Sample,
    Proposal,
    Draft,
    Candidate,
    Final,
    Deprecated,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConceptTemplate {
    #[serde(rename = "Definitions", skip_serializing_if = "Option::is_none")]
    pub definitions: Option<Definitions>,
    #[serde(rename = "Rules", skip_serializing_if = "Option::is_none")]
    pub rules: Option<Rules>,
    #[serde(rename = "SubTemplates", skip_serializing_if = "Option::is_none")]
    pub sub_templates: Option<SubTemplates>,
    #[serde(flatten)]
    pub identity: Identity,
    #[serde(rename = "@applicableSchema")]
    pub applicable_schema: XmlList,
    #[serde(rename = "@applicableEntity", skip_serializing_if = "Option::is_none")]
    pub applicable_entity: Option<XmlList>,
    #[serde(
        rename = "@isPartial",
        default,
        deserialize_with = "deserialize_optional_xsd_bool",
        skip_serializing_if = "Option::is_none"
    )]
    pub is_partial: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    #[serde(rename = "AttributeRule")]
    pub attribute_rules: Vec<AttributeRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubTemplates {
    #[serde(rename = "ConceptTemplate")]
    pub concept_templates: Vec<ConceptTemplate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttributeRule {
    #[serde(
        rename = "@xsi:nil",
        alias = "@nil",
        default,
        deserialize_with = "deserialize_optional_xsd_bool",
        skip_serializing_if = "Option::is_none"
    )]
    pub nil: Option<bool>,
    #[serde(rename = "EntityRules", skip_serializing_if = "Option::is_none")]
    pub entity_rules: Option<EntityRules>,
    #[serde(rename = "Constraints", skip_serializing_if = "Option::is_none")]
    pub constraints: Option<Constraints>,
    #[serde(rename = "@AttributeName")]
    pub attribute_name: String,
    #[serde(rename = "@RuleID", skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
    #[serde(rename = "@Description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityRules {
    #[serde(rename = "EntityRule")]
    pub entity_rules: Vec<EntityRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityRule {
    #[serde(rename = "References", skip_serializing_if = "Option::is_none")]
    pub references: Option<TemplateReference>,
    #[serde(rename = "AttributeRules", skip_serializing_if = "Option::is_none")]
    pub attribute_rules: Option<Rules>,
    #[serde(rename = "Constraints", skip_serializing_if = "Option::is_none")]
    pub constraints: Option<Constraints>,
    #[serde(rename = "@EntityName")]
    pub entity_name: String,
    #[serde(rename = "@RuleID", skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
    #[serde(rename = "@Description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateReference {
    #[serde(rename = "Template")]
    pub template: GenericReference,
    #[serde(rename = "@IdPrefix", skip_serializing_if = "Option::is_none")]
    pub id_prefix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Constraints {
    #[serde(rename = "Constraint")]
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Constraint {
    #[serde(rename = "@Expression")]
    pub expression: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelView {
    #[serde(rename = "Definitions", skip_serializing_if = "Option::is_none")]
    pub definitions: Option<Definitions>,
    #[serde(rename = "BaseView", skip_serializing_if = "Option::is_none")]
    pub base_view: Option<GenericReference>,
    #[serde(
        rename = "ExchangeRequirements",
        skip_serializing_if = "Option::is_none"
    )]
    pub exchange_requirements: Option<ExchangeRequirements>,
    #[serde(rename = "Roots", skip_serializing_if = "Option::is_none")]
    pub roots: Option<Roots>,
    #[serde(flatten)]
    pub identity: Identity,
    #[serde(rename = "@applicableSchema")]
    pub applicable_schema: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExchangeRequirements {
    #[serde(rename = "ExchangeRequirement")]
    pub exchange_requirements: Vec<ExchangeRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExchangeRequirement {
    #[serde(rename = "Definitions", skip_serializing_if = "Option::is_none")]
    pub definitions: Option<Definitions>,
    #[serde(rename = "@applicability", skip_serializing_if = "Option::is_none")]
    pub applicability: Option<Applicability>,
    #[serde(flatten)]
    pub identity: Identity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Applicability {
    Export,
    Import,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Roots {
    #[serde(rename = "ConceptRoot")]
    pub concept_roots: Vec<ConceptRoot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConceptRoot {
    #[serde(rename = "Definitions", skip_serializing_if = "Option::is_none")]
    pub definitions: Option<Definitions>,
    #[serde(rename = "Applicability", skip_serializing_if = "Option::is_none")]
    pub applicability: Option<ConceptApplicability>,
    #[serde(rename = "Concepts", skip_serializing_if = "Option::is_none")]
    pub concepts: Option<Concepts>,
    #[serde(flatten)]
    pub identity: Identity,
    #[serde(
        rename = "@applicableRootEntity",
        skip_serializing_if = "Option::is_none"
    )]
    pub applicable_root_entity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConceptApplicability {
    #[serde(rename = "Definitions", skip_serializing_if = "Option::is_none")]
    pub definitions: Option<Definitions>,
    #[serde(rename = "Template")]
    pub template: GenericReference,
    #[serde(rename = "TemplateRules")]
    pub template_rules: TemplateRules,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Concepts {
    #[serde(rename = "Concept")]
    pub concepts: Vec<Concept>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Concept {
    #[serde(rename = "Definitions", skip_serializing_if = "Option::is_none")]
    pub definitions: Option<Definitions>,
    #[serde(rename = "Template")]
    pub template: GenericReference,
    #[serde(rename = "Requirements", skip_serializing_if = "Option::is_none")]
    pub requirements: Option<Requirements>,
    #[serde(rename = "TemplateRules")]
    pub template_rules: TemplateRules,
    #[serde(flatten)]
    pub identity: Identity,
    #[serde(rename = "@baseConcept", skip_serializing_if = "Option::is_none")]
    pub base_concept: Option<Uuid>,
    #[serde(
        rename = "@override",
        default,
        deserialize_with = "deserialize_optional_xsd_bool",
        skip_serializing_if = "Option::is_none"
    )]
    pub override_base: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateRules {
    #[serde(rename = "$value", default)]
    pub children: Vec<TemplateRuleNode>,
    #[serde(rename = "@operator", skip_serializing_if = "Option::is_none")]
    pub operator: Option<LogicalOperator>,
    #[serde(rename = "@Description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateRuleNode {
    #[serde(rename = "TemplateRules")]
    Group(TemplateRules),
    #[serde(rename = "TemplateRule")]
    Rule(TemplateRule),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateRule {
    #[serde(rename = "@Parameters")]
    pub parameters: String,
    #[serde(rename = "@Description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogicalOperator {
    And,
    Or,
    Not,
    Nand,
    Nor,
    Xor,
    Nxor,
}

impl LogicalOperator {
    #[must_use]
    pub fn effective(operator: Option<Self>) -> Self {
        operator.unwrap_or(Self::And)
    }

    #[must_use]
    pub fn apply(self, values: &[bool]) -> bool {
        match self {
            Self::And => values.iter().all(|value| *value),
            Self::Or => values.iter().any(|value| *value),
            Self::Not => !values.iter().all(|value| *value),
            Self::Nand => !values.iter().all(|value| *value),
            Self::Nor => !values.iter().any(|value| *value),
            Self::Xor => values.iter().filter(|value| **value).count() == 1,
            Self::Nxor => values.iter().filter(|value| **value).count() != 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definitions {
    #[serde(rename = "Definition")]
    pub definitions: Vec<Definition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    #[serde(rename = "Body", skip_serializing_if = "Option::is_none")]
    pub body: Option<Body>,
    #[serde(rename = "Link", default)]
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Body {
    #[serde(rename = "$text", default)]
    pub text: String,
    #[serde(rename = "@lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(rename = "@tags", skip_serializing_if = "Option::is_none")]
    pub tags: Option<XmlList>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    #[serde(
        rename = "@xsi:nil",
        alias = "@nil",
        default,
        deserialize_with = "deserialize_optional_xsd_bool",
        skip_serializing_if = "Option::is_none"
    )]
    pub nil: Option<bool>,
    #[serde(rename = "@lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(rename = "@title", skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "@category", skip_serializing_if = "Option::is_none")]
    pub category: Option<LinkCategory>,
    #[serde(rename = "@href")]
    pub href: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkCategory {
    Definition,
    Agreement,
    Diagram,
    Instantiation,
    Example,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirements {
    #[serde(rename = "Requirement")]
    pub requirements: Vec<Requirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    #[serde(rename = "$text", default)]
    pub description: String,
    #[serde(rename = "@exchangeRequirement")]
    pub exchange_requirement: Uuid,
    #[serde(rename = "@requirement")]
    pub requirement: RequirementLevel,
    #[serde(rename = "@applicability", skip_serializing_if = "Option::is_none")]
    pub applicability: Option<Applicability>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequirementLevel {
    #[serde(rename = "mandatory")]
    Mandatory,
    #[serde(rename = "recommended")]
    Recommended,
    #[serde(rename = "not-relevant")]
    NotRelevant,
    #[serde(rename = "not-recommended")]
    NotRecommended,
    #[serde(rename = "excluded")]
    Excluded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenericReference {
    #[serde(rename = "@ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<Uuid>,
    #[serde(rename = "@href", skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

/// An XML Schema list value, represented canonically with one ASCII space.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct XmlList(pub Vec<String>);

impl XmlList {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Serialize for XmlList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.join(" "))
    }
}

impl<'de> Deserialize<'de> for XmlList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self(value.split_whitespace().map(str::to_owned).collect()))
    }
}

fn deserialize_optional_xsd_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    value
        .map(|value| match value.as_str() {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            _ => Err(serde::de::Error::custom(format!(
                "invalid XML Schema boolean `{value}`"
            ))),
        })
        .transpose()
}

macro_rules! serialize_identity {
    ($state:ident, $identity:expr) => {{
        $state.serialize_field("@uuid", &$identity.uuid)?;
        $state.serialize_field("@name", &$identity.name)?;
        if let Some(value) = &$identity.code {
            $state.serialize_field("@code", value)?;
        }
        if let Some(value) = &$identity.version {
            $state.serialize_field("@version", value)?;
        }
        if let Some(value) = &$identity.status {
            $state.serialize_field("@status", value)?;
        }
        if let Some(value) = &$identity.author {
            $state.serialize_field("@author", value)?;
        }
        if let Some(value) = &$identity.owner {
            $state.serialize_field("@owner", value)?;
        }
        if let Some(value) = &$identity.copyright {
            $state.serialize_field("@copyright", value)?;
        }
    }};
}

impl Serialize for MvdXml {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("mvdXML", 11)?;
        if let Some(value) = &self.templates {
            state.serialize_field("Templates", value)?;
        }
        if let Some(value) = &self.views {
            state.serialize_field("Views", value)?;
        }
        serialize_identity!(state, self.identity);
        state.end()
    }
}

impl Serialize for ConceptTemplate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ConceptTemplate", 14)?;
        if let Some(value) = &self.definitions {
            state.serialize_field("Definitions", value)?;
        }
        if let Some(value) = &self.rules {
            state.serialize_field("Rules", value)?;
        }
        if let Some(value) = &self.sub_templates {
            state.serialize_field("SubTemplates", value)?;
        }
        serialize_identity!(state, self.identity);
        state.serialize_field("@applicableSchema", &self.applicable_schema)?;
        if let Some(value) = &self.applicable_entity {
            state.serialize_field("@applicableEntity", value)?;
        }
        state.end()
    }
}

impl Serialize for ModelView {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ModelView", 14)?;
        if let Some(value) = &self.definitions {
            state.serialize_field("Definitions", value)?;
        }
        if let Some(value) = &self.base_view {
            state.serialize_field("BaseView", value)?;
        }
        if let Some(value) = &self.exchange_requirements {
            state.serialize_field("ExchangeRequirements", value)?;
        }
        if let Some(value) = &self.roots {
            state.serialize_field("Roots", value)?;
        }
        serialize_identity!(state, self.identity);
        state.serialize_field("@applicableSchema", &self.applicable_schema)?;
        state.end()
    }
}

impl Serialize for ExchangeRequirement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ExchangeRequirement", 10)?;
        if let Some(value) = &self.definitions {
            state.serialize_field("Definitions", value)?;
        }
        serialize_identity!(state, self.identity);
        state.serialize_field("@applicability", &self.applicability)?;
        state.end()
    }
}

impl Serialize for ConceptRoot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ConceptRoot", 13)?;
        if let Some(value) = &self.definitions {
            state.serialize_field("Definitions", value)?;
        }
        if let Some(value) = &self.applicability {
            state.serialize_field("Applicability", value)?;
        }
        if let Some(value) = &self.concepts {
            state.serialize_field("Concepts", value)?;
        }
        serialize_identity!(state, self.identity);
        if let Some(value) = &self.applicable_root_entity {
            state.serialize_field("@applicableRootEntity", value)?;
        }
        state.end()
    }
}

impl Serialize for Concept {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Concept", 14)?;
        if let Some(value) = &self.definitions {
            state.serialize_field("Definitions", value)?;
        }
        state.serialize_field("Template", &self.template)?;
        if let Some(value) = &self.requirements {
            state.serialize_field("Requirements", value)?;
        }
        state.serialize_field("TemplateRules", &self.template_rules)?;
        serialize_identity!(state, self.identity);
        if let Some(value) = &self.base_concept {
            state.serialize_field("@baseConcept", value)?;
        }
        if let Some(value) = &self.override_base {
            state.serialize_field("@override", value)?;
        }
        state.end()
    }
}
