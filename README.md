# Meerkat

Multi-tenant error tracking platform. Ingest application errors, deduplicate them into issues, and manage them across organizations and projects. Similar in purpose to Sentry.

## Stack

- **Backend:** Rust, Axum, PostgreSQL (sqlx), Tokio
- **Frontend:** Vue 3, TypeScript, TanStack Query, Tailwind CSS
- **Auth:** OIDC via [Keyline](https://github.com/yonasBSD/keyline) (self-hosted)

## Architecture

Vertical slice with DDD across six crates:

| Crate | Role |
|---|---|
| `meerkat-domain` | Aggregates, value objects, domain logic — no framework deps |
| `meerkat-application` | Commands, handlers, mediator pipeline (auth → rate limit → UoW) |
| `meerkat-infrastructure` | Postgres repositories, OIDC/JWKS providers, read stores |
| `meerkat-api` | Axum handlers, DTOs, auth middleware, OpenAPI docs |
| `meerkat-server` | Composition root, startup, CLI |
| `meerkat-macros` | Proc macros: `uuid_id!`, `slug_id!`, `Reconstitute` |

## Local Development

Start dependencies:

```bash
docker compose up -d
```

Run migrations and start the API:

```bash
cargo run -p meerkat-server -- migrate
cargo run -p meerkat-server -- api
```

Start the frontend:

```bash
cd meerkat-ui
pnpm install
pnpm dev        # http://localhost:5173
```

Default Keyline credentials: `admin@meerkat.local` / `meerkat`

## Configuration

Key environment variables for `meerkat-server`:

| Variable | Description |
|---|---|
| `MEERKAT_DATABASE_URL` | Postgres connection string |
| `MEERKAT_LISTEN_ADDR` | Default: `0.0.0.0:3030` |
| `MEERKAT_BASE_DOMAIN` | Root domain for tenant subdomain resolution |
| `MEERKAT_MASTER_ORG_NAME` / `_SLUG` | Bootstrap organization |
| `MEERKAT_MASTER_OIDC_*` | OIDC issuer, client ID, audience, discovery URL, claim mappings |

## Testing

```bash
cargo test                                         # all tests
cargo test -p meerkat-domain                       # domain only
cargo test -p meerkat-domain -- organization       # specific module
```

Integration tests use `.hurl` files co-located with handlers.

## Deployment

Both the backend and frontend have multi-stage Dockerfiles. The backend binary runs on port 8080; the frontend is served by nginx on port 80.
