---
name: Sentry envelope compatibility deferred
description: Decision to ship own JSON ingest format first; Sentry envelope compatibility layer is a separate future phase
type: project
---

Sentry envelope format compatibility is deferred — not part of Phase 2. Phase 2 ingest endpoint accepts only Meerkat's own JSON payload format.

**Why:** Keeps the ingestion pipeline simple and avoids coupling to Sentry's envelope spec before the core model is proven.

**How to apply:** Don't design Phase 2 ingest around Sentry envelope constraints. Compatibility layer gets its own phase later.
