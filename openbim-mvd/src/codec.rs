//! Secure, bounded XML parsing and canonical serialization.

use quick_xml::NsReader;
use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use thiserror::Error;

use crate::model::{MvdXml, NAMESPACE};

/// Resource limits applied before typed deserialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseLimits {
    pub max_input_bytes: usize,
    pub max_depth: usize,
    pub max_events: usize,
    pub max_text_bytes: usize,
}

impl Default for ParseLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 64 * 1024 * 1024,
            max_depth: 256,
            max_events: 2_000_000,
            max_text_bytes: 32 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Error)]
pub enum CodecError {
    #[error("input is {actual} bytes; limit is {limit}")]
    InputTooLarge { actual: usize, limit: usize },
    #[error("XML depth {actual} exceeds limit {limit}")]
    DepthLimit { actual: usize, limit: usize },
    #[error("XML event count exceeds limit {0}")]
    EventLimit(usize),
    #[error("XML text is {actual} bytes; limit is {limit}")]
    TextLimit { actual: usize, limit: usize },
    #[error("DTD declarations are forbidden")]
    DtdForbidden,
    #[error("expected mvdXML 1.1 root, found `{0}`")]
    WrongRoot(String),
    #[error("expected mvdXML namespace `{expected}`, found `{actual}`")]
    WrongNamespace {
        expected: &'static str,
        actual: String,
    },
    #[error("missing document root")]
    MissingRoot,
    #[error("XML is malformed: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("XML attribute is malformed: {0}")]
    Attribute(String),
    #[error("`{name}` must use the lowercase mvdXML UUID lexical form: `{value}`")]
    InvalidUuid { name: String, value: String },
    #[error("element `{child}` appears out of schema order inside `{parent}`")]
    ElementOrder { parent: String, child: String },
    #[error("attribute `{0}` is not declared on mvdXML")]
    UnknownRootAttribute(String),
    #[error("mvdXML does not match the typed 1.1 schema: {0}")]
    Decode(#[from] quick_xml::DeError),
    #[error("could not serialize mvdXML: {0}")]
    Encode(String),
}

impl MvdXml {
    /// Parses mvdXML 1.1 with conservative resource limits and no DTD support.
    pub fn from_xml(xml: &str) -> Result<Self, CodecError> {
        Self::from_xml_with_limits(xml, ParseLimits::default())
    }

    /// Parses mvdXML 1.1 with caller-selected resource limits.
    pub fn from_xml_with_limits(xml: &str, limits: ParseLimits) -> Result<Self, CodecError> {
        preflight(xml, limits)?;
        Ok(quick_xml::de::from_str(xml)?)
    }

    /// Serializes a canonical, namespace-qualified mvdXML 1.1 document.
    pub fn to_xml(&self) -> Result<String, CodecError> {
        let body = quick_xml::se::to_string(self)
            .map_err(|error| CodecError::Encode(error.to_string()))?;
        let root_end = body
            .find('>')
            .ok_or_else(|| CodecError::Encode("serializer emitted no root tag".into()))?;
        let mut output = String::with_capacity(body.len() + NAMESPACE.len() + 48);
        output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        output.push_str(&body[..root_end]);
        output.push_str(" xmlns=\"");
        output.push_str(NAMESPACE);
        output.push('"');
        output.push_str(" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"");
        output.push_str(&body[root_end..]);
        Ok(output)
    }
}

fn preflight(xml: &str, limits: ParseLimits) -> Result<(), CodecError> {
    if xml.len() > limits.max_input_bytes {
        return Err(CodecError::InputTooLarge {
            actual: xml.len(),
            limit: limits.max_input_bytes,
        });
    }

    let mut reader = NsReader::from_reader(xml.as_bytes());
    reader.config_mut().check_end_names = true;
    let mut buffer = Vec::new();
    let mut depth = 0usize;
    let mut events = 0usize;
    let mut text_bytes = 0usize;
    let mut saw_root = false;
    let mut stack: Vec<ElementFrame> = Vec::new();

    loop {
        let (namespace, event) = reader.read_resolved_event_into(&mut buffer)?;
        events = events.saturating_add(1);
        if events > limits.max_events {
            return Err(CodecError::EventLimit(limits.max_events));
        }

        match event {
            Event::Start(element) => {
                validate_uuid_attributes(&element)?;
                let name = element_local_name(&element);
                observe_child(&mut stack, &name)?;
                stack.push(ElementFrame { name, last_rank: 0 });
                if !saw_root {
                    verify_root(&element, namespace)?;
                    saw_root = true;
                } else {
                    verify_namespace(namespace)?;
                }
                depth = depth.saturating_add(1);
                if depth > limits.max_depth {
                    return Err(CodecError::DepthLimit {
                        actual: depth,
                        limit: limits.max_depth,
                    });
                }
            }
            Event::Empty(element) => {
                validate_uuid_attributes(&element)?;
                let name = element_local_name(&element);
                observe_child(&mut stack, &name)?;
                if !saw_root {
                    verify_root(&element, namespace)?;
                    saw_root = true;
                } else {
                    verify_namespace(namespace)?;
                }
            }
            Event::End(_) => {
                depth = depth.saturating_sub(1);
                stack.pop();
            }
            Event::Text(text) => {
                text_bytes = text_bytes.saturating_add(text.as_ref().len());
                if text_bytes > limits.max_text_bytes {
                    return Err(CodecError::TextLimit {
                        actual: text_bytes,
                        limit: limits.max_text_bytes,
                    });
                }
            }
            Event::CData(text) => {
                text_bytes = text_bytes.saturating_add(text.as_ref().len());
                if text_bytes > limits.max_text_bytes {
                    return Err(CodecError::TextLimit {
                        actual: text_bytes,
                        limit: limits.max_text_bytes,
                    });
                }
            }
            Event::DocType(_) => return Err(CodecError::DtdForbidden),
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }

    if saw_root {
        Ok(())
    } else {
        Err(CodecError::MissingRoot)
    }
}

fn verify_root(
    element: &quick_xml::events::BytesStart<'_>,
    namespace: ResolveResult<'_>,
) -> Result<(), CodecError> {
    let local_name = String::from_utf8_lossy(element.local_name().as_ref()).into_owned();
    if local_name != "mvdXML" {
        return Err(CodecError::WrongRoot(local_name));
    }
    verify_root_attributes(element)?;
    verify_namespace(namespace)
}

fn verify_namespace(namespace: ResolveResult<'_>) -> Result<(), CodecError> {
    let actual = match namespace {
        ResolveResult::Bound(value) => String::from_utf8_lossy(value.as_ref()).into_owned(),
        ResolveResult::Unbound => String::new(),
        ResolveResult::Unknown(prefix) => {
            format!(
                "unresolved prefix {}",
                String::from_utf8_lossy(prefix.as_ref())
            )
        }
    };
    if actual != NAMESPACE {
        return Err(CodecError::WrongNamespace {
            expected: NAMESPACE,
            actual,
        });
    }
    Ok(())
}

fn verify_root_attributes(element: &quick_xml::events::BytesStart<'_>) -> Result<(), CodecError> {
    for attribute in element.attributes().with_checks(false) {
        let attribute = attribute.map_err(|error| CodecError::Attribute(error.to_string()))?;
        let name = String::from_utf8_lossy(attribute.key.as_ref()).into_owned();
        let allowed = name == "xmlns"
            || name.starts_with("xmlns:")
            || name.ends_with(":schemaLocation")
            || matches!(
                name.as_str(),
                "uuid" | "name" | "code" | "version" | "status" | "author" | "owner" | "copyright"
            );
        if !allowed {
            return Err(CodecError::UnknownRootAttribute(name));
        }
    }
    Ok(())
}

fn validate_uuid_attributes(element: &quick_xml::events::BytesStart<'_>) -> Result<(), CodecError> {
    for attribute in element.attributes().with_checks(false) {
        let attribute = attribute.map_err(|error| CodecError::Attribute(error.to_string()))?;
        let local = attribute.key.local_name();
        if !matches!(
            local.as_ref(),
            b"uuid" | b"ref" | b"baseConcept" | b"exchangeRequirement"
        ) {
            continue;
        }
        let value = String::from_utf8_lossy(attribute.value.as_ref()).into_owned();
        let canonical = uuid::Uuid::parse_str(&value)
            .is_ok_and(|parsed| parsed.hyphenated().to_string() == value);
        if !canonical {
            return Err(CodecError::InvalidUuid {
                name: String::from_utf8_lossy(local.as_ref()).into_owned(),
                value,
            });
        }
    }
    Ok(())
}

struct ElementFrame {
    name: String,
    last_rank: usize,
}

fn element_local_name(element: &quick_xml::events::BytesStart<'_>) -> String {
    String::from_utf8_lossy(element.local_name().as_ref()).into_owned()
}

fn observe_child(stack: &mut [ElementFrame], child: &str) -> Result<(), CodecError> {
    let Some(parent) = stack.last_mut() else {
        return Ok(());
    };
    let Some(rank) = child_rank(&parent.name, child) else {
        return Ok(());
    };
    if rank < parent.last_rank {
        return Err(CodecError::ElementOrder {
            parent: parent.name.clone(),
            child: child.to_owned(),
        });
    }
    parent.last_rank = rank;
    Ok(())
}

fn child_rank(parent: &str, child: &str) -> Option<usize> {
    if parent == "TemplateRules" {
        return matches!(child, "TemplateRule" | "TemplateRules").then_some(0);
    }
    let order: &[&str] = schema_child_order(parent)?;
    order.iter().position(|name| *name == child)
}

fn schema_child_order(parent: &str) -> Option<&'static [&'static str]> {
    Some(match parent {
        "mvdXML" => &["Templates", "Views"],
        "Templates" | "SubTemplates" => &["ConceptTemplate"],
        "ConceptTemplate" => &["Definitions", "Rules", "SubTemplates"],
        "Definitions" => &["Definition"],
        "Definition" => &["Body", "Link"],
        "Rules" | "AttributeRules" => &["AttributeRule"],
        "AttributeRule" => &["EntityRules", "Constraints"],
        "EntityRules" => &["EntityRule"],
        "EntityRule" => &["References", "AttributeRules", "Constraints"],
        "References" => &["Template"],
        "Constraints" => &["Constraint"],
        "Views" => &["ModelView"],
        "ModelView" => &["Definitions", "BaseView", "ExchangeRequirements", "Roots"],
        "ExchangeRequirements" => &["ExchangeRequirement"],
        "ExchangeRequirement" => &["Definitions"],
        "Roots" => &["ConceptRoot"],
        "ConceptRoot" => &["Definitions", "Applicability", "Concepts"],
        "Applicability" => &["Definitions", "Template", "TemplateRules"],
        "Concepts" => &["Concept"],
        "Concept" => &["Definitions", "Template", "Requirements", "TemplateRules"],
        "Requirements" => &["Requirement"],
        _ => return None,
    })
}
