# Security

The parser treats mvdXML as untrusted data. Before typed deserialization it enforces the root and namespace, rejects DTD declarations, checks element order and canonical UUID spellings, and applies four resource budgets.

| Limit | Default |
|---|---:|
| Input bytes | 64 MiB |
| XML nesting depth | 256 |
| XML events | 2,000,000 |
| Aggregate text and CDATA | 32 MiB |

Applications should lower these defaults when expected documents are smaller. Parsing in a dedicated process remains appropriate for high-risk workloads; this library is not a sandbox.

Rule values and regular-expression patterns come from the document/application boundary. The evaluator performs no file, network, or IFC access. Validation diagnoses document state but does not establish provenance, verify signatures, or make trust decisions.

Report vulnerabilities through the private process in the repository [security policy](https://github.com/openbimrs/mvd/security/policy).
