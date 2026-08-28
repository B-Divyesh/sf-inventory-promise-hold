# Stock Promise — independent verification 2

Work order: `inventory-promise-hold-verify-2`
Candidate: `89db0bdb7520ce50b0a054f3a90a0ac64bb86c10`
Live URL: <https://inventory-promise-hold.sociobot.in>
Verified: 2026-08-28 UTC

## Verdict: FAIL

The source candidate is healthy in a clean, isolated single-instance run, and
the live frontend is the exact candidate. The deployed backend nevertheless
fails the product's central shared-inventory contract: it has no persistent
volume and can scale to three independent SQLite replicas. Fresh live evidence
reproduced both data loss and split-brain responses. This is release-blocking.

## Release-blocking defect

### QA-01 — Critical — live stock, holds, and sessions are split across ephemeral replicas

Expected: one durable SQLite database mounted at `/data`, with exactly one
replica, so every operator sees the same atomic hold ledger across scaling and
restarts.

Actual:

- Read-only Azure inspection of `sf-inventory-promise-hold` at ready revision
  `sf-inventory-promise-hold--0000011` returned `minReplicas: 1`,
  `maxReplicas: 3`, `volumes: null`, and `volumeMounts: null`.
- The prior repair handoff recorded a configured location, inventory, outcomes,
  and audit records on durable storage. At the start of this fresh run,
  `GET /api/status` returned `{"setup_required":true,...}` and the known QA PIN
  returned `409 Set up this location first.` The deployed data had disappeared.
- QA set up the otherwise-empty live instance and the complete live Playwright
  suite passed 5/5. A subsequent 1,000-request load smoke at 110.57 requests/s
  caused all three allowed replicas to run.
- Immediately afterward, 120 concurrent `GET /api/status` requests returned
  two contradictory states: 81 reported `setup_required:true`; 39 reported
  `setup_required:false`.
- A login attempt sequence against the same URL returned `409`, then `200`.
  Using the successful session token for 60 `GET /api/bootstrap` requests
  produced 19 successful responses and 41 `401` responses because the token
  exists in only one replica's database.
- All 30 sampled `/health` responses reported the candidate SHA, proving this
  is divergent state among replicas of the candidate deployment, not mixed
  application versions.

Impact: staff can be routed to different inventories, holds, audit logs, and
session stores. A hold created on one replica does not protect stock on another,
so the service can make the duplicate promises it is specifically meant to
prevent. New replicas also expose first-run setup again, and a restart can erase
all operational history.

Required repair: apply the checked-in durable-storage contract after the
generic deployment, confirm an Azure Files mount at `/data`, and pin both min
and max replicas to one. Restore the intended durable database if applicable.
Then repeat restart, authenticated load, and split-brain checks before release.

## Clean-checkout and build evidence

The tree began clean on `main`, with both `HEAD` and `origin/main` at the exact
candidate SHA. No product source was changed.

Environment: Node `22.23.2`, npm `10.9.8`, rustc/cargo `1.98.0`, Playwright
`1.58.2`.

- `npm ci` — passed; 139 packages installed, 0 vulnerabilities.
- `npm test` — passed: 1 Vitest test, 3 Node deployment-contract tests, and 9
  Rust unit/integration tests.
- `npm run check` — passed: 0 Svelte/TypeScript errors or warnings; Clippy
  passed for all targets with warnings denied.
- `cargo fmt --all -- --check` — passed.
- `npm run build` — passed and produced `dist/`.
- `BUILD_SHA=89db0bdb7520ce50b0a054f3a90a0ac64bb86c10 cargo build --release --locked`
  — passed; the resulting binary reported that exact SHA.
- A container engine is not installed in the verifier image, so a local
  `docker build` could not be run. The exact locked frontend and backend stages
  passed independently, and the live container reports the candidate SHA.

Production bundle sizes:

- JavaScript: 75,665 bytes raw / 28.01 KB gzip (budget: 200 KB).
- CSS: 18,457 bytes raw / 5.05 KB gzip (budget: 50 KB).
- Mobile environmental image: 15,414 bytes (budget: 300 KB).
- Entire `dist/`: 194,217 bytes.
- No web-font downloads; system fonts are used.

## Local functional and backend evidence

`npm run test:e2e` passed 5/5 against a fresh production build and database.
It covered first-run setup, an empty inventory, adding stock, creating and
converting a hold, CSV export, anonymous API denial, invalid-PIN recovery,
desktop keyboard use, 390 px layout, legal routes, cache/security headers,
reduced motion, service-worker update, offline reload, console errors, and axe.

Additional API checks against the release binary covered:

- Invalid PIN, duplicate setup, ambiguous SKU, negative and over-maximum stock,
  zero hold quantity, 4-minute and 481-minute durations, blank customer, and a
  missing inventory ID. Each returned the expected `400`, `401`, `404`, or
  `409`, with successful recovery afterward.
- The accepted upper stock boundary was 100,000,000. An exact 3-of-3 hold was
  accepted and conversion left on-hand/held/available at `0/0/0`.
- Two simultaneous 2-unit requests against 3 available units returned exactly
  one `201` and one `409`; the surviving view showed 2 held and 1 available.
- Lowering on-hand stock below an active hold returned `409`. Release restored
  availability, and a repeated resolution returned `409`.
- A due hold was expired on the next bootstrap, returned its unit to available
  stock, and recorded `resolved_by: Clock`.
- Audit entries for release, conversion, and expiry were present. The Rust test
  also proved the append-only trigger rejects audit deletion.
- CSV export returned the correct content type/header and preserved an order
  note containing a comma and quotes.
- Anonymous bootstrap, hold, audit, inventory, and export boundaries are
  guarded; login throttling has per-client, global, and concurrent-hash tests.

Persistence was tested by stopping and restarting the release binary on the
same file database. Instance ID
`fa7708d0-8fc9-4fe4-be56-a6340832c2cd`, location, 4 inventory rows, 4 hold rows,
and 12 audit rows survived unchanged. Startup correctly reported
`schema: existing (connection deferred)` and `instance_identity: existing`.

## Live browser, accessibility, privacy, and PWA evidence

After the missing live database was initialized solely for QA, the checked-in
live suite passed 5/5 on desktop (1440×1000) and mobile (390×844):

- Correct title, `lang=en`, one `<h1>`, one `<main>`, meaningful image alt text,
  and working legal deep links.
- Zero serious/critical axe findings on authenticated desktop and mobile.
- Zero browser console errors, page errors, or failed requests.
- Keyboard skip link and invalid-PIN recovery passed; dialog focus stayed
  contained. The 390 px view had no horizontal overflow.
- An additional 390 px authenticated settings audit found no serious/critical
  axe issues and no visible interactive target smaller than 44×44 px.
- `prefers-reduced-motion: reduce` was active; the stylesheet collapses all
  animation/transition duration to `.01ms` and one iteration.
- Service-worker registration/update passed, the page was controlled after
  reload, and offline reload showed the explicit offline recovery state.
- Browser requests during normal use remained same-origin. There are no
  analytics, tracking scripts, third-party fonts, or cookies. A cross-origin
  request with an attacker origin received no CORS allowance. License tokens
  are the only documented optional billing-API disclosure and are stored
  locally.
- Visual inspection confirmed the product-specific blue-hour stockroom system,
  clear hierarchy, readable mobile stacking, and visible non-color state copy.

The factory URL verifier passed in 598 ms with correct title/language/main/H1,
no missing alt text, no unlabeled buttons, and zero console errors.

Lighthouse 13 mobile:

- Performance 99
- Accessibility 100
- Best Practices 100
- SEO 100
- FCP 1.4 s, LCP 1.5 s, TBT 90 ms, CLS 0
- 95,868 transferred bytes, 6 requests, 0 third-party bytes

## Live response and identity evidence

- `/health` returned `200`, `no-store`, and build SHA
  `89db0bdb7520ce50b0a054f3a90a0ac64bb86c10`; 30/30 samples matched.
- `/`, `/privacy`, and `/terms` returned `200` with
  `no-cache, must-revalidate`.
- Hashed JS/CSS use one-year immutable caching; `sw.js` uses
  `no-cache, no-store, must-revalidate`; stable images use one-day caching.
- Responses include CSP with `frame-ancestors 'none'`, HSTS, nosniff,
  same-origin referrer policy, and a restrictive permissions policy.
- The load smoke completed 1,000 requests with 0 transport errors at 110.57
  requests/s: p50 21.5 ms, p95 279.0 ms, p99 319.6 ms, max 435.8 ms. Its
  successful scale-out exposed QA-01.

## Final state and limitations

The QA run created a location and one converted QA item on one ephemeral live
replica. Other replicas remained uninitialized; the service was deliberately
left untouched once the split-brain evidence was captured. No infrastructure,
DNS, billing, or product code was changed.

Library/CLI packaging is not applicable. This is a web application with a Rust
backend and PWA shell.
