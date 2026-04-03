# Phase 2 — Error Ingestion Pipeline

Sliced into small, vertically deliverable pieces. Each slice is independently testable and buildable.

---

## Slice 1 — Project Key Domain Model ✓

- `ProjectKeyId` typed ID
- `ProjectKey` aggregate — `KeyToken` (random hex), label, status (active/revoked), project ref, created_at
- `ProjectKey::generate()` constructor — creates random key token, takes `clock`
- `ProjectKey::revoke()` transition
- Domain tests

## Slice 2 — Project Key Persistence ✓

- `project_keys` table migration (project_id FK, key_token unique index, status, label, timestamps)
- `ProjectKeyRepository` port trait (add, save, find)
- Postgres repo impl (buffered entry pattern)
- `ProjectKeyPersistence` insert/update helpers
- Wired into `UnitOfWork` / `PgUnitOfWork`

## Slice 2b — Auto-Generate Key on Project Creation

Event-driven: react to `ProjectCreated` domain event in the same UoW, not coupled to `CreateProject` handler.

- Domain event handler for `ProjectCreated` → generates default key
- Include DSN string in project detail response (`{scheme}://{key_token}@{host}/{project_slug}`)
- `ProjectKeyReadStore` port trait (list by project)
- Handler tests

## Slice 3 — Event Value Objects

Domain building blocks for error events. No persistence yet.

- `EventId` typed ID
- `EventLevel` enum — fatal, error, warning, info, debug
- `Platform` enum — dotnet, go, javascript, other
- `StackFrame` value object — filename, function, lineno, colno, context_line
- `StackTrace` — vec of frames
- `ExceptionValue` — type, value, stack trace
- `TagPair` — key + value (bounded length)
- `Environment`, `Release` value objects (validated strings)
- Domain tests for each

## Slice 4 — Event Aggregate

- `Event` aggregate — message, level, platform, timestamp, server_name, exceptions (vec), tags, extra (json), environment, release, fingerprint (vec of strings)
- `Event::new()` — validates required fields
- Stores computed fingerprint hash (next slice supplies the algorithm)
- Domain tests

## Slice 5 — Issue Aggregate

- `IssueId` typed ID
- `IssueStatus` enum — unresolved, resolved, ignored
- `Issue` aggregate — title, fingerprint_hash, status, first_seen, last_seen, event_count, level, project ref
- `Issue::new()` — created from first event
- `Issue::record_event()` — bumps last_seen + event_count, escalates level if higher
- `Issue::resolve()`, `Issue::ignore()`, `Issue::reopen()`
- Domain tests

## Slice 6 — Fingerprinting Service

- Pure domain service / function
- Default: hash(exception_type + top N non-library frames) -> hex SHA-256
- Custom: if event supplies explicit fingerprint vec, use it directly
- Unit tests — same stack = same hash, different = different, custom overrides default

## Slice 7 — Event & Issue Persistence

- `events` table migration (JSONB for tags/extra/exceptions), `issues` table migration
- `EventRepository` port (save)
- `EventReadStore` port (get by id, list by issue)
- `IssueRepository` port (save, get_by_fingerprint)
- `IssueReadStore` port (list by project, get by id)
- Postgres repo impls
- Integration tests

## Slice 8 — Ingest Endpoint (Happy Path)

The money endpoint. Project key auth, no OIDC.

- Project key auth extractor — `X-Meerkat-Key` header or `?meerkat_key=` query param, resolves project from public key
- `IngestEvent` command + handler — fingerprint, find-or-create issue, save event
- `POST /api/v1/projects/:slug/store` — accepts simplified JSON payload
- Response: `{ "id": "<event_id>" }`
- Handler tests, hurl integration test

## Slice 9 — Payload Validation & Size Limits

- Max payload size (200KB default)
- Required fields, type checks, string length bounds
- Tag count limit (max 50)
- Extra data depth/size limit
- Typed validation errors with descriptive messages
- Tests for each rejection case

## Slice 10 — Project Key Management Endpoints

Now that ingestion works, let admins manage keys.

- `POST /api/v1/projects/:slug/keys` — create additional key
- `GET /api/v1/projects/:slug/keys` — list keys (public key + status + label)
- `DELETE /api/v1/projects/:slug/keys/:id` — revoke
- `CreateProjectKey`, `RevokeProjectKey` commands + handlers
- RBAC: project admin+
- Hurl integration tests

## Slice 11 — Per-Key Rate Limiting

- Rate limit config on `ProjectKey` (optional max events/min)
- In-memory sliding window counter, keyed by public key
- `429 Too Many Requests` + `Retry-After` header
- Default project-level limit when no per-key override
- Tests

---

## Dependency Graph

```
Slice 1 ──> Slice 2
Slice 3 ──> Slice 4 ──> Slice 5
                  \        |
           Slice 6 ───> Slice 7 ──> Slice 8 ──> Slice 9
                           ^                        |
Slice 2 (key auth) ────────┘              Slice 10 ──> Slice 11
```

Slices 1-2 (key track) and Slices 3-6 (domain model track) can be built **in parallel**. They converge at Slice 7-8. Slices 10-11 come after ingestion works.

## Unresolved Questions

- Sentry envelope compatibility — accept Sentry envelope format, or only our own JSON for now?
- Event retention — TTL / cleanup strategy in this phase, or defer?
- Async ingestion — synchronous write in Slice 8, or queue from the start?
- Stack frame "in-app" classification — distinguish library vs app frames in fingerprinting now, or defer?
