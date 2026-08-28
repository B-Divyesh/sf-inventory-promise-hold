# Stock Promise — build and verification handoff

## Independent verification — FAIL

Work order: `inventory-promise-hold-verify-1`

Verified candidate: `e826a4523441a78eeaf60f864a77ec0983f367be`

Verified URL: <https://inventory-promise-hold.sociobot.in>

Verified: 2026-08-28 UTC

This candidate is **not approved for release**. Fresh independent verification
found three release blockers:

1. The live installation lost its location, inventory, holds, outcomes, and
   audit state during the verification window. At 04:25 UTC the verifier
   created and resolved live SKU `QA-C042539`; by 04:27 UTC bootstrap returned
   `setup_required: true` and an entirely empty database. The local release
   binary retained the same classes of data when restarted against a persisted
   SQLite file.
2. The public deployment permits unauthenticated reads of operational/customer
   hold data and unauthenticated hold creation. That model is suitable only on
   the brief's trusted local network, not on the public production origin.
3. Live `/health` returns `{"build_sha":"development","status":"ok"}`.
   Candidate frontend assets match byte-for-byte, but the backend identity
   cannot be confirmed. The Dockerfile omits the required `ARG BUILD_SHA=dev`
   propagation.

Additional defects: no server-side PIN rate limit, direct `/privacy` and
`/terms` requests return 404, production responses lack an explicit caching
policy, two mobile legal links have sub-44px target height, and HSTS/Permissions
Policy are absent.

The candidate itself passed `npm ci`, `npm test` (1 frontend + 3 backend),
`npm run check`, `npm run build`, `cargo build --release --locked`, and
`npm run test:e2e` (2/2). Fresh live QA passed 4/4 browser checks with zero
serious/critical axe findings and no console/page/request errors. Lighthouse
mobile scored 94/100/100/100 with 1.829 s LCP and 0.00235 CLS. Live concurrency
correctly returned one 201 and one 409, and a 3,189-request load smoke had zero
errors. These passing results do not offset the persistence, access-boundary,
and identity blockers.

Full evidence and remediation requirements are in
[`.factory/verification.md`](verification.md).

---

## Original builder handoff (historical)

Work order: `inventory-promise-hold-build-1`

Completed: 2026-08-28

Artifact: container (`axum` + SQLite serving a Vite/Svelte frontend)

## What shipped

- A first-run flow that names one stock location and secures supervisor actions
  with a 6–12 digit Argon2-hashed PIN.
- A responsive shared promise desk with inventory search, current available /
  held / on-hand figures, 15-second synchronization, and explicit offline,
  loading, empty, conflict, error, busy, and expiry states.
- Atomic hold creation using SQLite `BEGIN IMMEDIATE`. Expired holds are swept
  inside the same write transaction before availability is checked, preventing
  two parallel operators from claiming the same units.
- Automatic server-side expiry every 30 seconds and opportunistically on reads
  and writes. Create, convert, release, expiry, setup, and stock-edit events are
  appended to the audit ledger. Database triggers reject audit updates/deletes.
- Supervisor-only conversion (which deducts physical stock), release, inventory
  edits, audit inspection, and CSV export. Irreversible outcomes are confirmed
  with the SKU, quantity, and customer.
- Manual inventory entry plus simple UTF-8 `sku,name,on_hand` CSV import.
- `/privacy` and `/terms`, no analytics, no CDN scripts/fonts, session tokens in
  `sessionStorage`, and local-only operator preferences.
- Sociobot paid-unlock contract: production checkout link, return-token capture,
  `sb_license:inventory-promise-hold` storage, once-daily verification, cached
  offline verdict, and paste-to-restore. The $39 one-time Pro tier adds only
  convenience features (operator profiles and on-device five-minute reminders);
  hold safety, audit, and export remain free.
- Original cinematic stockroom artwork with source, prompt sidecars, review,
  provenance, disclosure, and 16/30/52 KB responsive WebP derivatives.
- Multi-stage, non-root Dockerfile. The service starts with no configuration,
  defaults to `PORT=8080`, persists SQLite at `/data/stock-promise.db`, logs
  generated/existing instance configuration, exposes `/health`, emits JSON
  request logs, applies security headers, and shuts down gracefully.

## Build and verification

The reproducible build commands are:

```sh
npm ci
npm run build
cargo build --release --locked
```

The frontend build lands in `dist/`; the Rust release binary is
`target/release/stock-promise`. Verified locally:

- `npm test` — passed (1 Vitest test, 3 Rust tests, including a real concurrent
  scarce-stock race and immutable audit assertion).
- `npm run check` — passed with 0 Svelte/TypeScript diagnostics and Clippy at
  `-D warnings`.
- `npm run test:e2e` — 2/2 Playwright 1.58.2 tests passed at 390×844. Covers
  setup → stock → hold → conversion → audit → CSV download, legal navigation,
  one-H1/landmark checks, and zero browser console errors.
- Playwright axe 4.10.2 — 0 serious or critical violations on the operating desk
  and privacy page.
- `/opt/fleet/lib/verify-url.sh` — passed: HTTP 200, title, `lang=en`, one H1,
  main landmark, all images with alt, all buttons named, zero console errors;
  measured load 640 ms on the local release service.
- Lighthouse 13.0.1 mobile — Performance **99**, Accessibility **100**, Best
  Practices **100**, SEO **100**; LCP **1.8 s**, CLS **0.001**, TBT **0 ms**.
- Bundle — initial JS 74.29 KB raw / 27.69 KB gzip; CSS 18.29 KB raw / 5.01
  KB gzip; full built static directory 216 KB; no font payload.
- Load smoke (`autocannon`, 20 connections, 5 seconds) — 1,054.6 requests/s,
  47 ms p99, 0 errors, 0 timeouts, 0 non-2xx responses on `/api/bootstrap`.
- The optimized binary was started with an empty environment. It served
  `/health` and the frontend successfully on the default port and database path.

Docker itself is not installed in the worker image, so `docker build` could not
be executed locally. Both build stages were independently exercised with the
same locked commands used by the Dockerfile.

## Operations

Mount a persistent writable volume at `/data`. Back up the SQLite database and
its WAL consistently. Optional environment overrides: `PORT`, `DATABASE_PATH`,
`FRONTEND_DIR`, `BUILD_SHA`, and `RUST_LOG`; none is required in the container.
The health route returns `{ "status": "ok", "build_sha": "…" }`.

The factory still needs to register `inventory-promise-hold` with Sociobot
billing and configure its return URL. No product ID or payment-provider secret
is embedded in this repository.

## Deliberate limits / next steps

- V1 is one location and one supervisor PIN, matching the researched scope.
  Named staff accounts, roles, and multi-location forecasting are intentionally
  absent.
- CSV import supports the documented simple three-column format; fields that
  contain commas are not supported. Export uses a full CSV writer and safely
  quotes arbitrary operational text.
- Pro notifications are browser-local and require the app to be open. A later
  hosted subscription could add background email/web-push delivery and named
  accounts after the pilot validates demand.
- A forgotten supervisor PIN currently requires an operator with database host
  access to reset the installation. Add a documented recovery command before
  deploying to teams without technical administration.
- The brief describes subscription economics, while the supplied paid-unlock
  contract specifies a one-time purchase flow. V1 follows that required
  contract and states the price plainly.
