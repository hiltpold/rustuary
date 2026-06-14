# Release process

This file describes the intended release process. It will evolve as the project moves from library MVP to platform.

## Versioning

Use semantic versioning for published packages and APIs once public consumers exist.

- Patch: compatible bug fixes and documentation-only changes.
- Minor: compatible new functionality.
- Major: breaking API, schema, or result semantics changes.

Before stable releases, use `0.x.y` versions and document compatibility expectations in the changelog.

## Release checklist

Before tagging a release:

- [ ] `CHANGELOG.md` has an entry for the release.
- [ ] ADRs exist for major technology or architecture decisions.
- [ ] Data contracts and API contracts are updated.
- [ ] Golden tests pass and expected actuarial output changes are explained.
- [ ] Rust core checks pass.
- [ ] Python package checks pass when bindings are included.
- [ ] Backend checks pass when services are included.
- [ ] UI checks pass when workbench changes are included.
- [ ] Security and dependency scans have no untriaged critical issues.
- [ ] Release artifacts are reproducible from the tag.

## Artifact types

Possible artifacts:

- Rust crates
- Python wheels
- Docker images
- Go service binaries
- SvelteKit workbench build
- OpenAPI/protobuf/schema bundles
- Example datasets and golden outputs

## Rollback notes

For service releases, document:

- Database migration reversibility.
- Object-store result compatibility.
- API compatibility.
- Background job compatibility.
- How to disable a faulty worker or engine version.
