use openbim_mvd::{MvdXml, Severity};
use std::{fs, path::PathBuf};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/fixtures/mvdXML_V1-1-Final-Documentation.xml")
}

#[test]
fn official_local_fixture_parses_validates_and_round_trips() {
    let path = fixture();
    if !path.exists() {
        return;
    }
    let xml = fs::read_to_string(path).unwrap();
    let document = MvdXml::from_xml(&xml).unwrap();
    let errors: Vec<_> = document
        .validate()
        .into_iter()
        .filter(|issue| issue.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
    let written = document.to_xml().unwrap();
    let reparsed = MvdXml::from_xml(&written).unwrap();
    assert_eq!(reparsed, document);
}
