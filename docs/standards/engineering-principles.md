# Engineering principles

These principles guide implementation across Rust, Python, Go, TypeScript, and infrastructure code.

## Default style

- Prefer simple, explicit, typed code over clever abstractions.
- Prefer composition over inheritance-like hierarchies.
- Prefer small modules with clear boundaries.
- Prefer deterministic functions and immutable inputs where practical.
- Prefer domain types over unstructured maps, strings, or ad-hoc dictionaries.
- Keep side effects at the edges: IO, network, database, filesystem, logging, and clocks should not be mixed into core calculations.
- Make invalid states hard to represent.
- Return rich, actionable errors instead of panics or generic exceptions.
- Optimize after correctness, tests, and observability exist. Add complex optimization only when it is measured or clearly justified.

## SOLID as heuristics, not ceremony

Use SOLID ideas when they make the code easier to change and test:

- Single responsibility: functions and types should have one clear reason to change.
- Open/closed: prefer adding new methods or adapters without rewriting stable core logic.
- Liskov substitution: implementations behind an interface must preserve the interface contract.
- Interface segregation: expose small interfaces rather than large catch-all traits/classes.
- Dependency inversion: business logic should depend on abstractions or domain inputs, not concrete databases, HTTP clients, UI frameworks, or dataframe libraries.

Do not force object-oriented patterns where plain functions, enums, traits, or data transformations are clearer.

## Functional programming preferences

Functional style is preferred where it improves clarity:

- Pure functions for actuarial formulas.
- Explicit inputs and outputs.
- No hidden global state.
- Immutable data by default.
- Data-in, data-out transformations for calculation steps.
- Composable selectors, validators, and projectors.
- Exhaustive pattern matching for method variants and selection rules.

Pragmatic exceptions are fine for performance, ergonomics, or framework integration. When mutable state is used, keep it local, obvious, and tested.

## Layering rules

- Core calculation code should not know about UI, HTTP, databases, Python, object storage, or auth.
- Adapters convert external data shapes into domain types.
- Services orchestrate workflows but should not reimplement actuarial formulas.
- UI presents, explains, and captures decisions but should not own calculation semantics.

## Actuarial-specific rules

- Preserve intermediate calculations that explain outputs.
- Keep candidate method outputs separate from selected/booked outputs.
- Record assumptions, exclusions, overrides, tail factors, selected factors, and rationale.
- Favor reproducibility over convenience.
- Golden tests are part of the model documentation.
