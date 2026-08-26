#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
TEMP="$(mktemp -d)"
WORK="$TEMP/work"
PROBE_TARGET="${CARGO_TARGET_DIR:-/tmp/openbim-mvd-target}/mutation-probe"
BEFORE="$(sha256sum "$ROOT"/openbim-mvd/src/*.rs | sha256sum | cut -d' ' -f1)"
trap 'rm -rf "$TEMP"' EXIT

mkdir -p "$WORK"
cp "$ROOT/Cargo.toml" "$ROOT/Cargo.lock" "$WORK/"
cp -a "$ROOT/openbim-mvd" "$WORK/"

cat > "$WORK/openbim-mvd/tests/mutation_contract.rs" <<'RS'
use openbim_mvd::{CodecError, MvdXml, ParameterExpression, Severity};

const MINIMAL: &str = concat!(
    "<mvdXML xmlns=\"http://buildingsmart-tech.org/mvd/XML/1.1\" ",
    "uuid=\"00000000-0000-4000-8000-000000000001\" name=\"minimal\"/>"
);

#[test]
fn namespace_gate() {
    let xml = MINIMAL.replace("http://buildingsmart-tech.org/mvd/XML/1.1", "urn:mutation");
    assert!(matches!(MvdXml::from_xml(&xml), Err(CodecError::WrongNamespace { .. })));
}

#[test]
fn uuid_gate() {
    let xml = MINIMAL.replace(
        "00000000-0000-4000-8000-000000000001",
        "AAAAAAAA-AAAA-4AAA-8AAA-AAAAAAAAAAAA",
    );
    assert!(matches!(MvdXml::from_xml(&xml), Err(CodecError::InvalidUuid { .. })));
}

#[test]
fn unknown_element_gate() {
    let xml = MINIMAL.replace("/>", "><Surprise/></mvdXML>");
    assert!(matches!(
        MvdXml::from_xml(&xml),
        Err(CodecError::UnknownElement { .. })
    ));
}

#[test]
fn attribute_namespace_gate() {
    let xml = MINIMAL.replace(
        "/>",
        " xmlns:q=\"urn:q\"><Templates><ConceptTemplate uuid=\"00000000-0000-4000-8000-000000000002\" name=\"t\" applicableSchema=\"IFC4\" q:isPartial=\"true\"/></Templates></mvdXML>",
    );
    assert!(matches!(
        MvdXml::from_xml(&xml),
        Err(CodecError::WrongAttributeNamespace { .. })
    ));
}

#[test]
fn expression_depth_gate() {
    let expression = format!("{}A=1{}", "(".repeat(65), ")".repeat(65));
    assert!(ParameterExpression::parse(&expression).is_err());
}

#[test]
fn dtd_gate() {
    let xml = MINIMAL.replace("<mvdXML", "<!DOCTYPE mvdXML><mvdXML");
    assert!(matches!(MvdXml::from_xml(&xml), Err(CodecError::DtdForbidden)));
}

#[test]
fn keyref_gate() {
    let mut document = MvdXml::from_xml(include_str!("fixtures/authored-complete.mvdxml")).unwrap();
    let concept = &mut document.views.as_mut().unwrap().model_views[0]
        .roots.as_mut().unwrap().concept_roots[0]
        .concepts.as_mut().unwrap().concepts[0];
    concept.template.reference = Some(uuid::Uuid::from_u128(999));
    assert!(document.validate().iter().any(|issue| {
        issue.severity == Severity::Error && issue.code == "xsd.template_keyref"
    }));
}

#[test]
fn rule_id_scope_gate() {
    const XML: &str = "<mvdXML xmlns='http://buildingsmart-tech.org/mvd/XML/1.1' uuid='00000000-0000-4000-8000-000000000001' name='r'><Templates><ConceptTemplate uuid='00000000-0000-4000-8000-000000000002' name='t' applicableSchema='IFC4'><Rules><AttributeRule AttributeName='A' RuleID='R'><EntityRules><EntityRule EntityName='E' RuleID='R'/></EntityRules></AttributeRule></Rules></ConceptTemplate></Templates></mvdXML>";
    let document = MvdXml::from_xml(XML).unwrap();
    assert!(document.validate().iter().any(|issue| {
        issue.severity == Severity::Error && issue.code == "mvd.ambiguous_rule_id"
    }));
}

#[test]
fn expression_gate() {
    assert!(ParameterExpression::parse("_Name[Value]='Wall'").is_ok());
}
RS

# A persistent target reuses third-party dependencies, but stale mutation
# artifacts must never satisfy a clean baseline on a later invocation.
CARGO_TARGET_DIR="$PROBE_TARGET" cargo clean \
  --manifest-path "$WORK/Cargo.toml" -p openbim-mvd >/dev/null

run_test() {
  CARGO_TARGET_DIR="$PROBE_TARGET" cargo test     --manifest-path "$WORK/Cargo.toml"     --test mutation_contract "$1" -- --exact >"$TEMP/$1.log" 2>&1
}

mutate() {
  local label="$1" file="$2" old="$3" new="$4" test_name="$5"
  cp "$ROOT/$file" "$WORK/$file"
  OLD="$old" NEW="$new" python3 - "$WORK/$file" <<'PY'
import os
import sys
from pathlib import Path

path = Path(sys.argv[1])
source = path.read_text()
old = os.environ["OLD"]
new = os.environ["NEW"]
if source.count(old) != 1:
    raise SystemExit(f"mutation-probe: expected one mutation target in {path}, found {source.count(old)}")
path.write_text(source.replace(old, new, 1))
PY
  if run_test "$test_name"; then
    echo "mutation-probe: FAIL ($label mutation survived)" >&2
    return 1
  fi
  echo "mutation-probe: detected $label mutation"
  cp "$ROOT/$file" "$WORK/$file"
}

failures=0
for test_name in namespace_gate uuid_gate unknown_element_gate attribute_namespace_gate expression_depth_gate dtd_gate keyref_gate rule_id_scope_gate expression_gate; do
  if ! run_test "$test_name"; then
    echo "mutation-probe: FAIL (clean baseline $test_name does not pass)" >&2
    tail -n 30 "$TEMP/$test_name.log" >&2
    failures=$((failures + 1))
  fi
done

if [[ "$failures" -eq 0 ]]; then
  mutate "namespace" "openbim-mvd/src/codec.rs"     "if actual != NAMESPACE {" "if false {" namespace_gate || failures=$((failures + 1))
  mutate "UUID" "openbim-mvd/src/codec.rs"     "if !canonical {" "if false {" uuid_gate || failures=$((failures + 1))
  mutate "unknown element" "openbim-mvd/src/codec.rs"     "child_rank(&parent.name, child).ok_or_else(|| CodecError::UnknownElement {" "Some(parent.last_rank).ok_or_else(|| CodecError::UnknownElement {" unknown_element_gate || failures=$((failures + 1))
  mutate "attribute namespace" "openbim-mvd/src/codec.rs"     "if !namespace_is_valid {" "if false {" attribute_namespace_gate || failures=$((failures + 1))
  mutate "expression depth" "openbim-mvd/src/rules.rs"     "if self.depth >= 64 {" "if false {" expression_depth_gate || failures=$((failures + 1))
  mutate "DTD" "openbim-mvd/src/codec.rs"     "Event::DocType(_) => return Err(CodecError::DtdForbidden)," "Event::DocType(_) => {}" dtd_gate || failures=$((failures + 1))
  mutate "template keyref" "openbim-mvd/src/validation.rs"     "&& !self.templates.contains_key(&id)" "&& false" keyref_gate || failures=$((failures + 1))
  mutate "RuleID scope" "openbim-mvd/src/validation.rs"     "if !sibling_ids.insert(id.to_owned()) {" "if false {" rule_id_scope_gate || failures=$((failures + 1))
  mutate "expression metric" "openbim-mvd/src/rules.rs"     '"Value" => Ok(Metric::Value),' '"__mutation__" => Ok(Metric::Value),' expression_gate || failures=$((failures + 1))
fi

LEAK_ROOT="$TEMP/leakage"
mkdir -p "$LEAK_ROOT"
printf 'safe public source\n' > "$LEAK_ROOT/README.md"
REPO_ROOT="$LEAK_ROOT" python3 "$ROOT/scripts/check-leakage.py" >/dev/null
printf '%s%s xmlns:xs="http://www.w3.org/2001/XMLSchema"/>\n' '<xs:' 'schema' > "$LEAK_ROOT/renamed-payload.txt"
if REPO_ROOT="$LEAK_ROOT" python3 "$ROOT/scripts/check-leakage.py" >/dev/null 2>&1; then
  echo "mutation-probe: FAIL (renamed XSD payload survived)" >&2
  failures=$((failures + 1))
else
  echo "mutation-probe: detected standards-payload mutation"
fi

AFTER="$(sha256sum "$ROOT"/openbim-mvd/src/*.rs | sha256sum | cut -d' ' -f1)"
if [[ "$BEFORE" != "$AFTER" ]]; then
  echo "mutation-probe: FAIL (working Rust source changed)" >&2
  failures=$((failures + 1))
fi

if [[ "$failures" -ne 0 ]]; then
  exit 1
fi

echo "mutation-probe: PASS (nine Rust contracts and leakage checker observed failing; working source unchanged)"
