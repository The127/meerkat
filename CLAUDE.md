# Meerkat

Multi-tenant SaaS platform with OIDC-based authentication.

## Architecture

Vertical slice architecture with DDD. Four crates:

- `meerkat-domain` — aggregates, value objects, change tracking (no framework deps)
- `meerkat-application` — commands, handlers, mediator pattern, port traits
- `meerkat-infrastructure` — Postgres persistence, port implementations
- `meerkat-api` — Axum HTTP handlers, DTOs, middleware
- `meerkat-server` — composition root, startup
- `meerkat-macros` — proc macros (uuid_id, slug_id, Reconstitute)

## Domain Model Conventions

- Domain models have NO timestamp fields (`created_at`/`updated_at`) — timestamps are an audit/persistence concern
- The persistence layer sets timestamps via `Clock` in `PgUnitOfWork::save_changes()` — INSERT sets both, UPDATE sets `updated_at`
- Read stores query timestamps directly from the database into read models for API responses
- Constructors are pure domain logic — no `Clock` dependency
- Value objects validate on construction and are immutable (ClientId, Audience, Url, slugs)
- Status transitions return `Result` with typed errors — no panics for invalid input
- Aggregates panic only on invariant violations (e.g. no active OIDC config found)

## Testing

- Test naming: `given_X_then_Y` pattern (e.g. `given_empty_name_then_creation_fails`)
- All tests use `// arrange`, `// act`, `// assert` section comments
- Each test module has a helper (e.g. `test_org()`, `test_project()`, `test_config()`) returning the entity directly
- Use `mockall::automock` on traits for test doubles; hand-roll only when automock can't handle the signature
- HTTP DTOs use `XyzDto` suffix (e.g. `CreateOrganizationRequestDto`)
- Never commit without explicit user approval
- Never silently rename, rewrite, or change test behavior without asking

## Build & Test

```
cargo test                           # all tests
cargo test -p meerkat-domain         # domain tests only
cargo test -p meerkat-domain -- organization  # specific module
```

Integration tests use hurl files (`.hurl`) co-located with handlers.
