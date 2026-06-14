# Monorepo map

## `engines/`

Calculation engines. Start with Rust only. A future C++ engine, GPU module, or external adapter can live here without changing product code.

## `bindings/`

Language bindings and SDKs that expose engine capabilities. Python is the first-class actuarial interface.

## `services/`

Backend services. Keep application orchestration out of the engine.

## `apps/`

User-facing products. The first app is a reserving workbench.

## `contracts/`

API contracts and schemas shared across services, UI, SDKs, and engine adapters.

## `data/`

Only synthetic examples and golden fixtures. Do not commit production client data.

## `docs/`

Architecture decisions, standards, runbooks, and product notes.

## `.agents/`

Repository-scoped reusable skills for coding agents. These are intentionally short and targeted.
