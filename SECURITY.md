# Security policy

## Supported versions

No public release is supported yet. Security fixes target `main` while the crate remains at pre-release version `0.1.0`.

## Reporting

Use GitHub's private vulnerability reporting for `openbimrs/mvd` when available. Otherwise contact the openbimrs maintainers privately. Do not publish exploit details, private fixtures, or standards material in an issue.

## Security posture

- Default parser limits are 64 MiB of input, 256 XML levels, 2,000,000 events, and 32 MiB of aggregate text/CDATA.
- The parser requires the mvdXML 1.1 root namespace, rejects DTD declarations, and does not fetch network resources.
- Canonical lowercase UUID syntax and schema child ordering are checked before typed deserialization.
- The typed model rejects unknown fields rather than silently executing extension content.
- Rule regular expressions use Rust's `regex` engine; values are supplied by the caller and are not read from IFC or the network.
- No XSD, PDF, credential, or official standards fixture is embedded in source or intended artifacts.

This crate does not provide process isolation, XML digital-signature verification, formal XSD validation, IFC trust decisions, or IFC graph traversal. Treat all input as untrusted data and lower `ParseLimits` for application-specific workloads.
