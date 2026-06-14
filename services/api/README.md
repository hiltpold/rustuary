# API service

Go backend placeholder for:

- API gateway
- auth and RBAC
- reserving workflow state
- job orchestration
- metadata
- audit trail
- exports

This service should not contain actuarial calculations. It should call the Rust engine through a worker, sidecar, process invocation, FFI, or service boundary depending on the production architecture decision.
