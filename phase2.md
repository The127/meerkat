# Phase 2 — Error Ingestion Pipeline

---

## Done

### Project Key Domain & Persistence
- `ProjectKey` aggregate with `KeyToken`, label, status (active/revoked), optional `RateLimit`
- `project_keys` table, repository, persistence helpers, wired into UoW
- Auto-generate default key on project creation via `ProjectCreated` domain event

### Project Key Management
- `POST/GET/DELETE /api/v1/projects/:slug/keys` endpoints with RBAC
- `POST /api/v1/projects/:slug/keys/:id/rate-limit` — set/clear per-key rate limit
- UI: Client Keys page (create, revoke, copy DSN, status filter, pagination, inline rate limit editing)

### Event & Issue Domain Models
- `Event` — immutable record (message, level, platform, timestamp, exception info, tags, extra, fingerprint hash)
- `Issue` — mutable aggregate (title, fingerprint hash, status, level, event count, first/last seen, version)
- `EventLevel` enum (fatal/error/warning/info/debug) with severity ordering
- `IssueStatus` enum (unresolved/resolved/ignored) with transition guards
- Level escalation (only goes up), out-of-order timestamp protection

### Fingerprinting
- `FingerprintService` port trait with `Sha256FingerprintService` implementation
- SHA-256 of `exception_type:exception_value`, falls back to message hash

### Event & Issue Persistence
- `events` and `issues` table migrations
- `EventRepository` (add), `IssueRepository` (find by fingerprint, add, save)
- `IssueReadStore` (list by project with status/search filtering)

### Ingest Endpoint
- `POST /api/v1/ingest` with project key auth (`X-Meerkat-Key` header)
- Computes fingerprint, finds-or-creates issue, saves event
- Routed through mediator pipeline (not bypassed)
- Returns `201 Created` with event ID

### Per-Key Rate Limiting
- `RateLimitBehavior` mediator behavior with DashMap fixed-window counters
- `RateLimitKey` extension (opt-in, only on IngestEvent)
- Per-key override via `RateLimit` value object (validated > 0), falls back to system default (1000/min)
- Returns 429 with `Retry-After` header when exceeded
- Revoked keys cannot have rate limits set

### Issues UI
- `GET /api/v1/projects/:slug/issues` endpoint with status/search filtering
- Issues page with filter tabs (all/unresolved/resolved/ignored)
- Level and status badges, event counts, relative timestamps
- "Send demo event" button
- Sidebar navigation (Issues, Client Keys, Settings)

### Code Quality
- Extracted `RoleValues` value object from role-value triplet
- `From`/`TryFrom` impls for ClaimMapping DTO conversions
- `org_extensions`/`project_extensions` helpers for auth boilerplate
- `ChangeBuffer`/`ChangeTracker` generics for repository pattern
- Decomposed `authenticate_inner`, `save_changes`, `run_api`
- `AppState` split into `AuthState`/`TenantState` sub-groups
- `MasterOidcConfig` extracted from `MeerkatConfig`
- `MkPagination` component and `usePagination` composable

---

## Remaining

### Issue Management
- Resolve/reopen/ignore actions from the issues page
- Backend: commands + handlers for status transitions

### Issue Detail Page
- Click an issue to see its individual events
- Event list with exception info, tags, extra context
- Backend: event read store (list by issue)

### Payload Validation & Size Limits
- Max payload size (200KB default)
- Required fields, type checks, string length bounds
- Tag count limit (max 50), extra data depth/size limit
- Typed validation errors with descriptive messages

### Rich Event Data
- `StackFrame` — filename, function, lineno, colno, context_line
- `StackTrace` — vec of frames
- Structured exception model (replace flat exception_type/exception_value strings)
- Update fingerprinting to use top N app frames

---

## Unresolved Questions

- Sentry envelope compatibility — accept Sentry envelope format, or only our own JSON?
- Event retention — TTL / cleanup strategy, or defer?
- Async ingestion — synchronous write for now, or queue from the start?
- Stack frame "in-app" classification — distinguish library vs app frames in fingerprinting?
