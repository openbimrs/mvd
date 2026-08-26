use openbim_mvd::{MvdXml, Severity};

fn source() -> MvdXml {
    MvdXml::from_xml(include_str!("fixtures/authored-complete.mvdxml")).unwrap()
}

fn has(document: &MvdXml, code: &str) -> bool {
    document
        .validate()
        .iter()
        .any(|issue| issue.severity == Severity::Error && issue.code == code)
}

#[test]
fn catches_identity_and_key_reference_defects() {
    let mut duplicate = source();
    let template_uuid = duplicate.templates.as_ref().unwrap().concept_templates[0]
        .identity
        .uuid;
    let views = duplicate.views.as_mut().unwrap();
    views.model_views[0].identity.uuid = template_uuid;
    assert!(
        has(&duplicate, "xsd.unique_uuids"),
        "{:#?}",
        duplicate.validate()
    );

    let mut missing = source();
    let views = missing.views.as_mut().unwrap();
    let roots = views.model_views[0].roots.as_mut().unwrap();
    let concept = &mut roots.concept_roots[0].concepts.as_mut().unwrap().concepts[0];
    concept.template.reference = Some(uuid::Uuid::from_u128(999));
    assert!(has(&missing, "xsd.template_keyref"));

    let mut exchange = source();
    let views = exchange.views.as_mut().unwrap();
    let roots = views.model_views[0].roots.as_mut().unwrap();
    let concept = &mut roots.concept_roots[0].concepts.as_mut().unwrap().concepts[0];
    concept.requirements.as_mut().unwrap().requirements[0].exchange_requirement =
        uuid::Uuid::from_u128(998);
    assert!(has(&exchange, "xsd.exchange_keyref"));
}

#[test]
fn catches_duplicate_rule_ids_across_a_template_rule_tree() {
    const XML: &str = "<mvdXML xmlns='http://buildingsmart-tech.org/mvd/XML/1.1' uuid='00000000-0000-4000-8000-000000000001' name='r'><Templates><ConceptTemplate uuid='00000000-0000-4000-8000-000000000002' name='t' applicableSchema='IFC4'><Rules><AttributeRule AttributeName='A' RuleID='R'><EntityRules><EntityRule EntityName='E' RuleID='R'/></EntityRules></AttributeRule></Rules></ConceptTemplate></Templates></mvdXML>";
    let document = MvdXml::from_xml(XML).unwrap();
    assert!(has(&document, "mvd.ambiguous_rule_id"));
}

#[test]
fn catches_cycles_unknown_parameters_and_invalid_cardinality() {
    let mut base_cycle = source();
    let views = base_cycle.views.as_mut().unwrap();
    let roots = views.model_views[0].roots.as_mut().unwrap();
    let concept = &mut roots.concept_roots[0].concepts.as_mut().unwrap().concepts[0];
    concept.base_concept = Some(concept.identity.uuid);
    assert!(has(&base_cycle, "mvd.base_concept_cycle"));

    let mut unknown = source();
    let views = unknown.views.as_mut().unwrap();
    let roots = views.model_views[0].roots.as_mut().unwrap();
    let concept = &mut roots.concept_roots[0].concepts.as_mut().unwrap().concepts[0];
    let rule = &mut concept.template_rules.children[0];
    let openbim_mvd::TemplateRuleNode::Rule(rule) = rule else {
        panic!()
    };
    rule.parameters = "Missing[Value]='x'".into();
    assert!(
        has(&unknown, "mvd.unknown_rule_id"),
        "{:#?}",
        unknown.validate()
    );

    let mut empty = source();
    empty.views.as_mut().unwrap().model_views.clear();
    assert!(has(&empty, "xsd.min_occurs"));
}
