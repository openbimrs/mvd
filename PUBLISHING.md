# Publishing and provenance

`openbim-mvd` is published on crates.io from `.github/workflows/release.yml` through crates.io trusted publishing (OIDC). No crates.io token is stored in the repository. As a 0.x crate it makes no API compatibility guarantee between minor versions.

## Standards-material boundary

The implementation is licensed under AGPL-3.0-or-later. Redistribution rights for locally obtained mvdXML schemas, PDFs, and official example documents are not assumed. They are not runtime or build dependencies and must not appear in the repository, crate archive, or documentation site.

- `references/` is ignored and reserved for local material.
- `tests/fixtures/mvdXML_V1-1-Final-Documentation.xml` is ignored and used only by an optional local test.
- `openbim-mvd/tests/fixtures/authored-complete.mvdxml` is independently authored, non-normative, and included in the crate package.
- `scripts/check-leakage.py` scans source names and bytes, archives, and the generated Pages tree.

## Release checklist

1. Choose the version, bump `version` in `openbim-mvd/Cargo.toml` and the root `[workspace.package]`, run `cargo update -p openbim-mvd`, and date the `## [Unreleased]` changelog section as `## [x.y.z] - YYYY-MM-DD` with its compare link.
2. Run the complete gate with Rust 1.85:

   ```bash
   export CARGO_TARGET_DIR=/tmp/openbim-mvd-target
   ./scripts/gate.sh
   ```

   The gate packages the crate and runs `scripts/check-leakage.py` on the resulting `.crate`. To inspect the contents by hand:

   ```bash
   cargo package -p openbim-mvd --allow-dirty --list
   python3 scripts/check-leakage.py "$CARGO_TARGET_DIR/package/openbim-mvd-<version>.crate"
   cargo publish -p openbim-mvd --dry-run
   ```

3. Merge the release commit to `main` through a reviewed pull request.
4. Push an annotated tag `vx.y.z` on that commit of `main`.
5. Approve the `crates.io` environment deployment when the workflow asks for it.
6. Confirm the package on crates.io, the API documentation on docs.rs, and the GitHub release.

## Release workflow

`.github/workflows/release.yml` runs on tags matching `v[0-9]*`:

- **gate** refuses a tag that is not on `main`, does not match the `openbim-mvd` version, or has no `CHANGELOG.md` section, then runs `./scripts/gate.sh` on the tagged commit (including the `.crate` leakage scan).
- **publish** runs in the protected `crates.io` GitHub environment (required reviewer; deployments only from `main` and version tags) with `id-token: write`. It skips publishing when the version is already live on crates.io, otherwise obtains a short-lived token through `rust-lang/crates-io-auth-action` and runs `cargo publish --locked`.
- **github-release** creates the GitHub release `vx.y.z` from the changelog section.

Re-running a partly failed release skips what is already live. Running the workflow by hand (Actions -> Release -> Run workflow) with an existing tag rehearses a release: the tagged commit is gated and packaged, nothing is published.

crates.io trusts this repository (`openbimrs/mvd`), the workflow file name `release.yml`, and the environment `crates.io` (crate Settings -> Trusted Publishing). Renaming either requires the same change on crates.io.

### First version of a new crate

Trusted publishing cannot create a crate. The first version (`0.1.0`) is therefore published by hand by a crate owner with a personal token from a clean checkout of the reviewed `main` commit, after the gate and leakage checks above. The owner then adds the trusted publisher on crates.io (repository `openbimrs/mvd`, workflow `release.yml`, environment `crates.io`) and pushes the tag `v0.1.0`; the workflow sees the version is already live and only creates the GitHub release. Every later version is published by the workflow.
