# Phase 2 — Error Ingestion Pipeline

---

## Done

### Project Key Domain & Persistence
- `ProjectKey` aggregate with `KeyToken`, label, status (active/revoked)
- `project_keys` table, repository, persistence helpers, wired into UoW
- Auto-generate default key on project creation via `ProjectCreated` domain event

### Project Key Management
- `POST/GET/DELETE /api/v1/projects/:slug/keys` endpoints with RBAC
- UI: Client Keys card on project settings page (create, revoke, copy DSN, status filter)

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
- Returns `201 Created` with event ID

### Issues UI
- `GET /api/v1/projects/:slug/issues` endpoint with status/search filtering
- Issues page with filter tabs (all/unresolved/resolved/ignored)
- Level and status badges, event counts, relative timestamps
- "Send demo event" button
- Sidebar navigation (Issues + Settings)

---

## Remaining

### Payload Validation & Size Limits
- Max payload size (200KB default)
- Required fields, type checks, string length bounds
- Tag count limit (max 50), extra data depth/size limit
- Typed validation errors with descriptive messages
- Tests for each rejection case

### Rich Event Value Objects
- `StackFrame` — filename, function, lineno, colno, context_line
- `StackTrace` — vec of frames
- `ExceptionValue` — type, value, stack trace (replace flat exception_type/exception_value strings)
- `TagPair` — key + value with bounded length validation
- `Environment`, `Release` validated string value objects
- Update fingerprinting to use top N app frames instead of just exception type+value

### Per-Key Rate Limiting
- Rate limit config on `ProjectKey` (optional max events/min)
- In-memory sliding window counter, keyed by public key
- `429 Too Many Requests` + `Retry-After` header
- Default project-level limit when no per-key override

### Issue Management UI
- Resolve/reopen/ignore actions from the issues page
- Issue detail page showing individual events

---

## Unresolved Questions

- Sentry envelope compatibility — accept Sentry envelope format, or only our own JSON?
- Event retention — TTL / cleanup strategy, or defer?
- Async ingestion — synchronous write for now, or queue from the start?
- Stack frame "in-app" classification — distinguish library vs app frames in fingerprinting?
