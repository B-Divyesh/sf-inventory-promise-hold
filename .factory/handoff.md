# Stock Promise — repair handoff

Work order: `inventory-promise-hold-repair-1`

Verifier report: `bf61612ef8c9013f6c74f1991a8b7bd19cf1d478`

Failed candidate: `e826a4523441a78eeaf60f864a77ec0983f367be`

Live product: <https://inventory-promise-hold.sociobot.in>

Completed: 2026-08-28 UTC

## Disposition

All release-blocking and additional findings in `.factory/verification.md` are
repaired with direct regression coverage. The researched brief, single-location
scope, blue-hour design system, atomic hold behavior, immutable audit ledger,
paid-unlock behavior, and original imagery are preserved.

## Repairs

### Durable shared state

- Root cause: the generic Container Apps template mounted no volume at `/data`
  and allowed three replicas, so each replica used a different ephemeral SQLite
  file.
- Added `deploy/ensure-persistent-data.sh`. It creates/reuses the product Azure
  Files share, mounts it at `/data`, pins min/max replicas to one, waits for the
  exact mounted revision, and reports the applied topology.
- Snapshotted the live verifier database before cutover, checkpointed it, and
  restored it to the durable share. Snapshot SHA-256:
  `495b659004b79ae542091a1a640f412ccc98e5eb65724524318fa168e4bbdd88`.
  The restored state contained the original instance ID, location, four SKUs,
  three outcomes, and eleven audit entries.
- Existing mounted databases now bind the listener with a lazy pool, make no
  startup journal/schema mutation, and delay the expiry sweep for 15 seconds.
  This lets Azure's replacement replica pass readiness before the old replica
  drains, without opening SQLite concurrently during the handoff.
- Live rolling-restart proof: replacement replica became ready with zero
  restarts, the old replica drained, and pre/post bootstrap location,
  inventory, active holds, and outcomes compared equal. A second clean restart
  also replaced the replica with zero restarts.
- Regression coverage reopens a file-backed database and verifies location,
  inventory, audit, and instance identity; deployment contracts assert the
  Azure Files mount, one-replica topology, lazy existing-database startup, and
  absence of a startup journal-mode mutation.

### Staff access boundary and PIN abuse controls

- `GET /api/bootstrap` and `POST /api/holds` now require a current bearer
  session. All inventory, customer/order references, operator names, notes,
  holds, and outcomes are withheld before PIN entry.
- Added public `GET /api/status`, which exposes only first-run readiness and
  server time so the access gate loads without a deliberate 401 console error.
- Added a semantic, keyboard-operable staff gate. Locking clears the in-memory
  operational view as well as the tab-scoped token.
- `/api/session` now permits at most 10 attempts per validated proxy client and
  30 installation-wide per minute, with at most four concurrent Argon2 checks.
  Excess requests return `429` and `Retry-After: 60`.
- Unit and browser regressions verify anonymous denial, authenticated recovery,
  per-client/global/concurrent limits, and no operational text after locking.

### Identity, routes, caching, and hardening

- `ARG BUILD_SHA=dev` is declared and propagated through the Docker build and
  runtime. The Rust binary also embeds it as the no-env fallback. `/health`
  returns the exact factory source commit.
- Direct `GET`/`HEAD /privacy` and `/terms` return `200`.
- API and health responses are `no-store`; HTML revalidates; `sw.js` is
  `no-cache, no-store, must-revalidate`; Vite-hashed JS/CSS are one-year
  `immutable`; stable images/icons have a one-day policy.
- Added HSTS and Permissions Policy while preserving CSP, no-sniff, referrer,
  and same-origin CORS behavior.
- Mobile footer legal links now expose at least `44×44` CSS px targets.
- Service-worker cache version advanced to `stock-promise-shell-v2`.

## Verification evidence

Environment: Node 22.23.2, npm 10.9.8, Rust/Cargo 1.98.0, Playwright 1.58.2.

### Clean/local gates

- `npm ci` — passed; 139 packages, 0 vulnerabilities.
- `npm test` — passed: 1 Vitest, 3 Node deployment contracts, 9 Rust
  unit/integration tests.
- `npm run check` — passed: 0 Svelte/TypeScript warnings or errors; Clippy
  passed with `-D warnings`.
- `npm run build` — passed and produced `dist/`.
- `BUILD_SHA=local-verification cargo build --release --locked` — passed.
- `npm run test:e2e` — 5/5 passed. Coverage includes full setup/stock/hold/
  conversion/export, anonymous API denial, desktop keyboard and PIN-error
  recovery, 390 px geometry/touch targets, axe, legal deep links, response
  policy, reduced motion, service-worker update, and offline recovery.
- Factory URL verifier against the release binary — passed in 657 ms with
  title, `lang=en`, one H1/main, complete alt/button names, and zero console
  errors.
- Local Lighthouse 13 mobile — Performance 99, Accessibility 100, Best
  Practices 100, SEO 100; LCP 1,804 ms, CLS 0.00071, TBT 100 ms.
- Bundle: JS 75,670 bytes raw / 28.01 KB gzip; CSS 18,457 bytes raw / 5.05
  KB gzip; mobile image 15,414 bytes; full `dist/` 194,007 bytes.
- Docker is unavailable in the worker image. The same locked stages were built
  locally, and ACR built and ran the multi-stage image successfully.

### Live gates

- Code-bearing verified revision: `66f8c4f577eaedda9ae0b294e43f9877b2f7eb2c`.
  `/health` returned that exact 40-character SHA before the final handoff-only
  commit. The final deployment is required to match its own repository HEAD by
  the same assertion; no source changes are permitted afterward.
- Live Playwright — 5/5 passed: desktop and 390 px mobile, keyboard, invalid-PIN
  recovery, hold conversion and CSV, zero serious/critical axe findings, zero
  console/page/request errors, privacy origin check, reduced motion,
  service-worker update/offline reload, anonymous denial, legal status, cache
  policy, hardening headers, and exact build identity.
- Live factory URL verifier — HTTP 200, 598 ms, semantic checks passed, zero
  console errors.
- Live Lighthouse 13 mobile — Performance 100, Accessibility 100, Best
  Practices 100, SEO 100; LCP 1,397 ms, FCP 1,351 ms, CLS 0, TBT 0 ms.
- Authenticated live load smoke — 3,000 requests in 5.04 seconds, 539.21
  requests/s average, 93 ms p99, 96 ms maximum, no reported errors/timeouts.
- Final live state after QA: one mounted replica, zero active holds, preserved
  original data plus two isolated repair-verification SKUs/outcomes created by
  the live conversion checks.

## Run and deploy

```sh
npm ci
npm test
npm run check
npm run build
npm run test:e2e
cargo build --release --locked

/opt/fleet/lib/deploy-container.sh inventory-promise-hold . Dockerfile 8080
deploy/ensure-persistent-data.sh inventory-promise-hold
```

After deployment, assert that `/health.build_sha` equals `git rev-parse HEAD`,
the template has min/max replicas `1`, and `/data` is mounted from
`data-inventory-promise-hold`.

## Known operational constraint

The factory's generic container deploy template intentionally knows nothing
about product storage and replaces the volume stanza. Always run the checked-in
`deploy/ensure-persistent-data.sh` immediately after the generic deploy. Stock
Promise is intentionally one SQLite-backed replica; do not raise the replica
count without first moving shared state to PostgreSQL.
