---
layout: home

hero:
  name: openbim-mvd
  text: Typed mvdXML, without pretending to execute IFC
  tagline: Pure-Rust mvdXML 1.1 parsing, deterministic writing, rule evaluation, and document validation behind explicit resource and standards boundaries.
  image:
    src: /logo.svg
    alt: openbim-mvd mark
  actions:
    - theme: brand
      text: Get started
      link: /guide/getting-started
    - theme: alt
      text: Capability matrix
      link: /capabilities
    - theme: alt
      text: View on GitHub
      link: https://github.com/openbimrs/mvd

features:
  - title: Strict typed documents
    details: Model the fixed mvdXML 1.1 vocabulary, namespace, ordering, identities, references, and cardinalities as Rust data.
  - title: Defensive XML handling
    details: Apply byte, depth, event, and text budgets before typed deserialization, with no DTD processing or network access.
  - title: Complete rule language
    details: Parse and evaluate parameter comparisons, metrics, regexes, groups, connectives, and nested TemplateRules against values supplied by an adapter.
  - title: Honest execution boundary
    details: Validate MVD documents and rule inputs without claiming IFC graph traversal, concept execution, generic XSD validation, or bundled standards assets.
---

## Scope at a glance

`openbim-mvd` is a Rust library for mvdXML 1.1 documents. It can parse, write, validate, and evaluate rule trees when the caller supplies values. It does **not** open IFC models, traverse entity graphs, extract template parameters, or decide whether an IFC model satisfies an MVD.

The project is pre-release at `0.1.0`. Begin with [getting started](/guide/getting-started), then use the [capability matrix](/capabilities) to check each boundary before integrating it.
