use openbim_mvd::{CodecError, Metric, MvdXml, ParameterValue, ParseLimits, RuleValues, Severity};

const MINIMAL: &str = concat!(
    "<mvdXML xmlns=\"http://buildingsmart-tech.org/mvd/XML/1.1\" ",
    "uuid=\"00000000-0000-4000-8000-000000000001\" name=\"minimal\"/>"
);

#[test]
fn authored_complete_document_is_schema_shaped_and_canonical() {
    let source = include_str!("fixtures/authored-complete.mvdxml");
    let document = MvdXml::from_xml(source).unwrap();
    let errors: Vec<_> = document
        .validate()
        .into_iter()
        .filter(|issue| issue.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");

    let canonical = document.to_xml().unwrap();
    let reparsed = MvdXml::from_xml(&canonical).unwrap();
    assert_eq!(reparsed, document);
    assert!(canonical.contains("xmlns=\"http://buildingsmart-tech.org/mvd/XML/1.1\""));
}

#[test]
fn rejects_wrong_namespace_doctype_and_noncanonical_uuids() {
    let wrong_namespace = MINIMAL.replace("http://buildingsmart-tech.org/mvd/XML/1.1", "urn:wrong");
    assert!(matches!(
        MvdXml::from_xml(&wrong_namespace),
        Err(CodecError::WrongNamespace { .. })
    ));

    let doctype = MINIMAL.replace("<mvdXML", "<!DOCTYPE mvdXML [<!ENTITY x 'boom'>]><mvdXML");
    assert!(matches!(
        MvdXml::from_xml(&doctype),
        Err(CodecError::DtdForbidden)
    ));

    let uppercase = MINIMAL.replace(
        "00000000-0000-4000-8000-000000000001",
        "AAAAAAAA-AAAA-4AAA-8AAA-AAAAAAAAAAAA",
    );
    assert!(matches!(
        MvdXml::from_xml(&uppercase),
        Err(CodecError::InvalidUuid { .. })
    ));

    let unknown_attribute = MINIMAL.replace(" name=", " surprise=\"x\" name=");
    assert!(matches!(
        MvdXml::from_xml(&unknown_attribute),
        Err(CodecError::UnknownRootAttribute(_))
    ));
}

#[test]
fn rejects_wrong_child_namespace_and_schema_sequence() {
    let wrong_ns = MINIMAL.replace("/>", "><x:Templates xmlns:x=\"urn:wrong\"/></mvdXML>");
    assert!(matches!(
        MvdXml::from_xml(&wrong_ns),
        Err(CodecError::WrongNamespace { .. })
    ));

    let sequence = MINIMAL.replace(
        "/>",
        "><Views><ModelView uuid=\"00000000-0000-4000-8000-000000000002\" name=\"v\" applicableSchema=\"IFC4\"/></Views><Templates><ConceptTemplate uuid=\"00000000-0000-4000-8000-000000000003\" name=\"t\" applicableSchema=\"IFC4\"/></Templates></mvdXML>",
    );
    assert!(matches!(
        MvdXml::from_xml(&sequence),
        Err(CodecError::ElementOrder { .. })
    ));

    let definition_order = MINIMAL.replace("/>", "><Templates><ConceptTemplate uuid=\"00000000-0000-4000-8000-000000000004\" name=\"t\" applicableSchema=\"IFC4\"><Definitions><Definition><Link href=\"urn:test\"/><Body>late</Body></Definition></Definitions></ConceptTemplate></Templates></mvdXML>");
    assert!(matches!(
        MvdXml::from_xml(&definition_order),
        Err(CodecError::ElementOrder { .. })
    ));

    let empty_definition = MINIMAL.replace("/>", "><Templates><ConceptTemplate uuid=\"00000000-0000-4000-8000-000000000005\" name=\"t\" applicableSchema=\"IFC4\"><Definitions><Definition/></Definitions></ConceptTemplate></Templates></mvdXML>");
    let document = MvdXml::from_xml(&empty_definition).unwrap();
    assert!(document.is_valid());

    let unknown_root_child = MINIMAL.replace("/>", "><Surprise/></mvdXML>");
    assert!(matches!(
        MvdXml::from_xml(&unknown_root_child),
        Err(CodecError::UnknownElement { .. })
    ));

    let unexpected_root_text = MINIMAL.replace("/>", ">surprise</mvdXML>");
    assert!(matches!(
        MvdXml::from_xml(&unexpected_root_text),
        Err(CodecError::UnexpectedText { .. })
    ));

    let unknown_nested_child =
        MINIMAL.replace("/>", "><Templates><Surprise/></Templates></mvdXML>");
    assert!(matches!(
        MvdXml::from_xml(&unknown_nested_child),
        Err(CodecError::UnknownElement { .. })
    ));
}

#[test]
fn enforces_input_and_depth_budgets() {
    let tiny = ParseLimits {
        max_input_bytes: 10,
        ..ParseLimits::default()
    };
    assert!(matches!(
        MvdXml::from_xml_with_limits(MINIMAL, tiny),
        Err(CodecError::InputTooLarge { .. })
    ));

    let nested = MINIMAL.replace("/>", "><Templates><ConceptTemplate uuid=\"00000000-0000-4000-8000-000000000002\" name=\"x\" applicableSchema=\"IFC4\"><SubTemplates><ConceptTemplate uuid=\"00000000-0000-4000-8000-000000000003\" name=\"y\" applicableSchema=\"IFC4\"/></SubTemplates></ConceptTemplate></Templates></mvdXML>");
    let shallow = ParseLimits {
        max_depth: 2,
        ..ParseLimits::default()
    };
    assert!(matches!(
        MvdXml::from_xml_with_limits(&nested, shallow),
        Err(CodecError::DepthLimit { .. })
    ));
}

#[test]
fn accepts_both_xml_schema_boolean_lexical_forms() {
    let source = include_str!("fixtures/authored-complete.mvdxml")
        .replace(
            "name=\"Named wall\"",
            "name=\"Named wall\" baseConcept=\"00000000-0000-4000-8000-000000000007\" override=\"1\"",
        )
        .replace("applicableEntity=\"IfcWall\"", "applicableEntity=\"IfcWall\" isPartial=\"0\"");
    let document = MvdXml::from_xml(&source).unwrap();
    assert_eq!(
        document.templates.as_ref().unwrap().concept_templates[0].is_partial,
        Some(false)
    );
    let views = document.views.unwrap();
    let concept = &views.model_views[0].roots.as_ref().unwrap().concept_roots[0]
        .concepts
        .as_ref()
        .unwrap()
        .concepts[0];
    assert_eq!(concept.override_base, Some(true));
}

#[test]
fn rejects_wrongly_qualified_attributes_and_nilled_content() {
    let qualified_model_attribute = MINIMAL.replace(
        "/>",
        " xmlns:q=\"urn:evil\"><Templates><ConceptTemplate uuid=\"00000000-0000-4000-8000-000000000002\" name=\"t\" applicableSchema=\"IFC4\" q:isPartial=\"true\"/></Templates></mvdXML>",
    );
    assert!(matches!(
        MvdXml::from_xml(&qualified_model_attribute),
        Err(CodecError::WrongAttributeNamespace { .. })
    ));

    let wrong_schema_location = MINIMAL.replace(
        "/>",
        " xmlns:q=\"urn:evil\" q:schemaLocation=\"urn:a urn:b\"/>",
    );
    assert!(matches!(
        MvdXml::from_xml(&wrong_schema_location),
        Err(CodecError::WrongAttributeNamespace { .. })
    ));

    let unqualified_nil = MINIMAL.replace(
        "/>",
        "><Templates><ConceptTemplate uuid=\"00000000-0000-4000-8000-000000000002\" name=\"t\" applicableSchema=\"IFC4\"><Rules><AttributeRule nil=\"true\" AttributeName=\"A\"/></Rules></ConceptTemplate></Templates></mvdXML>",
    );
    assert!(matches!(
        MvdXml::from_xml(&unqualified_nil),
        Err(CodecError::WrongAttributeNamespace { .. })
    ));

    let nilled_content = MINIMAL.replace(
        "/>",
        " xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"><Templates><ConceptTemplate uuid=\"00000000-0000-4000-8000-000000000002\" name=\"t\" applicableSchema=\"IFC4\"><Rules><AttributeRule xsi:nil=\"true\" AttributeName=\"A\"><Constraints><Constraint Expression=\"A=1\"/></Constraints></AttributeRule></Rules></ConceptTemplate></Templates></mvdXML>",
    );
    assert!(matches!(
        MvdXml::from_xml(&nilled_content),
        Err(CodecError::NilledElementHasContent { .. })
    ));
}

#[test]
fn preserves_schema_nillable_attributes() {
    let source = include_str!("fixtures/authored-complete.mvdxml")
        .replace("xmlns=\"http://buildingsmart-tech.org/mvd/XML/1.1\"", "xmlns=\"http://buildingsmart-tech.org/mvd/XML/1.1\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"")
        .replace("<Link href=", "<Link xsi:nil=\"true\" href=");
    let document = MvdXml::from_xml(&source).unwrap();
    let link = &document.templates.as_ref().unwrap().concept_templates[0]
        .definitions
        .as_ref()
        .unwrap()
        .definitions[0]
        .links[0];
    assert_eq!(link.nil, Some(true));
    MvdXml::from_xml(&document.to_xml().unwrap()).unwrap();
}

#[test]
fn evaluates_a_complete_nested_template_rule_group() {
    let document = MvdXml::from_xml(include_str!("fixtures/authored-complete.mvdxml")).unwrap();
    let views = document.views.unwrap();
    let roots = views.model_views[0].roots.as_ref().unwrap();
    let concept = &roots.concept_roots[0].concepts.as_ref().unwrap().concepts[0];
    let mut values = RuleValues::new();
    values.insert(
        "_Name",
        Some(Metric::Value),
        ParameterValue::String("Wall".into()),
    );
    values.insert(
        "_Name",
        Some(Metric::Exists),
        ParameterValue::Logical(openbim_mvd::rules::LogicalValue::False),
    );
    assert!(concept.template_rules.evaluate(&values).unwrap());
}
