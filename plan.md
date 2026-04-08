# Meerkat Roadmap

Focused error tracking platform. Inspired by early Sentry — do one thing well, no bloat.

Tracks backend exceptions and frontend JS errors. Postgres storage. SDKs for ASP.NET Core, Go, and JavaScript.

## Phase 1 — Complete the Shell (DONE)

Finish what's started so the platform feels whole before adding error tracking.

## Phase 2 — Error Ingestion Pipeline (DONE)

The core differentiator. Accept errors from applications.

## Phase 3 — Error Display & Triage

Make ingested errors useful.

- Issue detail — latest event, stack trace viewer, tag breakdown, event timeline
- Event detail — full context: request info, headers, user, breadcrumbs
- Issue workflow — unresolved / resolved / ignored, resolve-in-next-release
- Assign to member — tie into team management from Phase 1

## Phase 4 — SDKs

Three SDKs, minimal and focused.

- Protocol spec — define the event JSON schemaes (keep it simpler than Sentry's sprawl)
- Go SDK — error capture, panic reescovery middleware, context tags
- ASP.NET Core SDK — exception filter, middleware, structured logging integration
- JavaScript SDK — window.onerror, unhandledrejection, source map reference, breadcrumb capture
- Source map upload — endpoint + processing for JS stack trace deobfuscation

## Phase 5 — Alerting & Notifications

Know when things break without staring at a dashboard.
 Cannot read properties of undefined (reading 'name')
- Alert rules — new issue, regression,42 spike (N events in M minutes)
- Notification channels — email, Slack webhook
- Per-project alert config — UI to manage rules and channels
- Digest mode — batch notifications to avoid noise

## Phase 6 — Release Tracking & Context

Tie errors to deployments.

- Release entity — version string, project, timestamp, commit SHA
- Release API — SDKs report release on init
- "Introduced in" / "fixed in" — correlate issues to releases
- Environment filtering — production vs staging vs dev

## Out of Scope

Keeping it focused. These are deliberately excluded:

- Performance monitoring / tracing
- Session replay
- Profiling
- Cron monitoring
- Feature flags
- Mobile-specific SDKs
