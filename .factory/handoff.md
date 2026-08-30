# Stock Promise — repair handoff

Work order: `inventory-promise-hold-repair-3`

Failed candidate: `fb0f1d3f582807c357342488794a1a53e453c93a`

Completed: 2026-08-30 UTC

## Repair summary

This repair addresses every finding in independent verification 3.

- Added `.factory/claims.json`, six observable claim tests, `/demo`, a shipped
  sample stockroom, persistent demo banner, reset, isolated
  `demo:stock-promise:state` storage, and `.factory/demo.md`.
- Production authentication now uses Sociobot Entra External ID with JWT
  signature, issuer, audience, tenant, expiry, and role validation. `staff`
  can bootstrap and create holds; `supervisor` alone can edit inventory,
  resolve holds, export, inspect audit data, set retention, or erase a
  location. Local PIN mode is explicitly development/test-only.
- Added a global per-client API limiter keyed to the first `X-Forwarded-For`
  hop. Reads allow 80/minute and writes 20/minute; auth remains stricter.
  Every limited response includes `429` and `Retry-After`; health stays exempt.
- Replaced inaccurate hosted-data copy. Supervisors can set 30–730 day
  redaction of resolved customer references, notes, and operator names, and
  can erase a whole location. New immutable audit events omit those values.
- Replaced the pinned Rust image with `rust:1-slim`, made SQLite migrations
  safe for an existing `/data` database, and changed the release path to ask
  the factory deployer for work-order `/data` storage before verifying only the
  `sf-inventory-promise-hold` one-ready-replica topology.
- Added route titles, canonical/social metadata, manifest, robots, sitemap,
  a styled HTTP 404, standard landing sections, accurate footer identity, and
  `.factory/copy-audit.md`.

## Failure reproduced

Before release, a target-only topology query for
`sf-inventory-promise-hold` returned ready revision
`sf-inventory-promise-hold--0000019` with `minReplicas: 1`,
`maxReplicas: 3`, `volumes: null`, and no `/data` mount. This is the reported
split-state root cause. The new topology verifier rejects that exact shape.

## Local verification

Environment: Node 22, npm 10, Rust stable, Playwright 1.58.2.

- `npm ci` — passed, 141 packages, 0 vulnerabilities.
- `npm test` — passed: Vitest, 6 deployment contracts, 13 Rust tests.
- `npm run check` — passed: Svelte diagnostics and strict Clippy.
- `cargo fmt --all -- --check`, `git diff --check`, and `bash -n deploy/*.sh`
  — passed.
- `npm run build` — passed; initial JS is 33.19 KB gzip, CSS 5.47 KB gzip.
- Exact inherited clean build command — passed:
  `BUILD_SHA=fb0f1d3f582807c357342488794a1a53e453c93a cargo build --release --locked`.
- `npm run test:e2e` — 8/8 passed: landing/demo, lifecycle, mobile 390 px,
  keyboard, legal routes, service-worker update, offline demo reload, and
  Playwright axe checks on landing, app, and legal pages with no serious or
  critical violations.
- `verify-url.sh` against the local release binary — passed: 579 ms load, no
  console errors, title/lang/one h1/main/alt/button checks passed.
- Local rate smoke: 100 concurrent `/health` requests returned 100×200. From
  one forwarded IP, 90 `/api/status` requests returned 80×200 and 10×429 with
  `Retry-After: 59`; 30 invalid setup writes returned 20×400 and 10×429 with
  the same header.
- Lighthouse mobile against the local release binary: Performance 99,
  Accessibility 100, Best Practices 100, SEO 100; LCP 1,651 ms; CLS 0.
- CIAM discovery check passed against the configured shared tenant and returned
  its issuer, authorization endpoint, and JWKS URL. The external axe CLI could
  not launch Selenium Chrome in this image; the checked-in Playwright AxeBuilder
  suite is the successful accessibility evidence.

## Run and verify

```sh
npm ci
npm test
npm run check
cargo fmt --all -- --check
npm run build
npm run test:e2e
BUILD_SHA=fb0f1d3f582807c357342488794a1a53e453c93a cargo build --release --locked
```

Run every command in `.factory/claims.json` from a clean clone. For a release:

```sh
npm run deploy
```

The release command refuses dirty worktrees, asks the factory deployment
configuration for `/data`, verifies one ready target replica and mounted Azure
Files storage, then compares `/health.build_sha` to the committed source.

## Needs operator action / known limits

The shared CIAM app must have
`https://inventory-promise-hold.sociobot.in/auth/callback` registered and its
operators assigned `staff` or `supervisor` app roles. Discovery was verified,
but this repair worker did not read CIAM tenant settings or use a human account
to complete a live login. No other release-blocking gaps are known.

## Live release evidence

Pending final deployment of this committed repair. This section is updated with
the deployed revision, exact build SHA, durable topology, public consistency
sample, and live verification after `npm run deploy` succeeds.
