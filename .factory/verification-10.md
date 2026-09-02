# Independent verification 10 — FAIL

Verified candidate: `ff809d81cef840ec4f4e13e6387018728c1d69f5`  
Live URL: <https://inventory-promise-hold.sociobot.in>  
Verification date: 2026-09-02

## Verdict

**FAIL.** The candidate and live deployment pass the product, claim, privacy,
accessibility, security, rate-limit, build-identity, and performance checks.
One mandatory backend runtime requirement is not met: a process started with
only `PORT` emits no startup configuration line unless `RUST_LOG=info` is also
supplied.

Defect count: **0 blocker, 0 high, 1 medium, 0 low**.

## Release-blocking finding

### MEDIUM — default startup suppresses the required configuration record

The backend contract requires the container to start with only `PORT` and log
one line saying which configuration was generated or supplied. A clean
candidate-stamped release binary was started as an unprivileged user with
`env -i PORT=4190`. `/health` returned 200 with the exact candidate SHA and a
fallback SQLite database was created, but the complete stdout/stderr log was
**0 bytes** after graceful shutdown.

The control run with `PORT=4191 RUST_LOG=info` emitted the intended structured
record, including `database_source:"default"`, `schema:"migrated"`,
`instance_identity:"generated"`, and `auth_mode:"ciam"`. This proves the
record exists but is hidden by the default tracing filter. The service must
make INFO the default when `RUST_LOG` is absent; requiring that extra variable
violates the factory's minimal runtime contract.

## First-read and demo gate

The cold live page passes. It says “Hold scarce stock before it is promised
twice,” names distributors and resellers taking orders in parallel, and puts
**Try it with sample data** above the fold beside “Open a sample stockroom.”
That one click opened `/?demo=1` with three SKUs, one active hold, and the
persistent “Demo — sample data, nothing is saved” banner with Reset demo and
Leave demo controls.

## Claims gate

`.factory/claims.json` exists with 22 entries. After the required clean
`npm ci`, every exact listed command passed independently:

- 11/11 browser claims: demo isolation and reset, no tracking, browser/session
  storage, CSV export, offline demo, Pro behavior, checkout status, and free
  core features.
- 11/11 Rust claims: CIAM defaults and authorization boundary, first setup,
  durable shared storage, role boundary, throttling, retention, expiry,
  contested-stock concurrency, append-only audit, and location erasure.

For chronology, the commands were also invoked before dependency installation,
as the work order requested. The 11 npm-backed commands could not start because
the clean clone did not yet contain `node_modules` (`vite: not found`); the 11
Rust commands passed. This was a harness precondition, not a failed product
assertion. The complete post-install claims run passed.

## Build and automated checks

- `npm ci`: passed; 143 packages installed, 0 vulnerabilities.
- `npm test`: passed — 3 Vitest, 9 Node contract, and 20 Rust tests.
- `npm run check`: passed — 0 Svelte/TypeScript diagnostics and warning-denied
  Clippy clean.
- `cargo fmt --all -- --check`: passed.
- `BUILD_SHA=ff809d81cef840ec4f4e13e6387018728c1d69f5 npm run build`: passed and
  produced `dist/`.
- `BUILD_SHA=ff809d81cef840ec4f4e13e6387018728c1d69f5 cargo build --release --locked`:
  passed.
- `npm run test:e2e:all`: passed — 21 normal browser tests and 1 hosted-auth
  browser test.
- Docker could not be invoked because neither Docker nor Podman is installed
  in this verification container. The Dockerfile contract tests and exact
  locked release builds passed.

## Functional and backend evidence

- The local full browser flow created a location, added stock, created and
  converted a hold, exported CSV, and verified supervisor locking.
- The live isolated demo rejected quantity `0` with “Value must be greater than
  or equal to 1” and `999` with “Value must be less than or equal to 9,” without
  changing holds. Recovery with a valid hold worked.
- A separate live boundary flow held all 9 available units, showed Fully held,
  released the hold, restored availability to 9, and recorded a released
  outcome. CSV contained its header and sample outcome rows; Reset demo restored
  the original three SKUs and one hold.
- Backend tests independently proved competing concurrent hold requests accept
  exactly one winner, timed expiry releases stock and records an outcome,
  durable data survives a database reopen, authorization boundaries hold, and
  the audit record rejects deletion.
- Live unauthenticated reads of bootstrap, audit, export, and retention all
  returned 401. Live unauthenticated inventory/hold writes and location erase
  also returned 401 without exposing or changing operational data.
- `/health` returned
  `{"build_sha":"ff809d81cef840ec4f4e13e6387018728c1d69f5","status":"ok"}`.
  `/api/auth/config` returned `{"mode":"ciam"}`.
- The real sign-in action redirected only to
  `https://sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650`
  with client `25c704f4-465a-47af-80ab-2c489466b697` and the product callback.
- Read allowance observed: 80 requests per client per 60 seconds; request 81
  returned 429 with `Retry-After: 57`.
- Write allowance observed: 20 requests per client per 60 seconds; request 21
  returned 429 with `Retry-After: 59`. The limiter wraps every `/api` route;
  `/health` is intentionally exempt.
- A 100-request concurrent read smoke from distinct client identities completed
  in 393 ms with 100/100 responses at 200; `/health` remained 200 afterward.

## Privacy, accessibility, PWA, and response policy

- A fresh home-to-demo workflow made requests only to the product origin.
  The browser held no cookies. Normal desktop and mobile flows produced no
  console errors or page errors.
- Live Axe scans found 0 serious/critical findings on home, demo, privacy,
  terms, and the designed 404 route.
- At 390 px there was no horizontal overflow, all visible controls measured at
  least 44 px, and 200% text retained the 390 px document width.
- Keyboard checks found the skip link first with a visible 3 px focus outline.
  Dialog focus stayed inside, Escape closed the dialog, and focus returned to
  its opener. Reduced-motion preference was active.
- The service worker controlled the demo, updated to
  `/sw.js?v=ff809d81cef840ec4f4e13e6387018728c1d69f5`, and the sample reloaded
  offline.
- HTML routes return `no-cache, must-revalidate`; hashed assets return one-year
  immutable caching; `/sw.js` and API/health responses use `no-store` policies.
  CSP is delivered as a response header with `frame-ancestors 'none'`; HSTS,
  nosniff, same-origin referrer policy, and restrictive Permissions-Policy are
  present.
- The link crawl found 200 responses for all real HTTP links. The deliberately
  requested unknown route correctly returned the designed 404 page.

## Deployment match and budgets

The live HTML, CSS, entry JS, and shared JS byte-match the candidate production
build. Startup JavaScript is 104.87 KB gzip total; CSS is 5.92 KB gzip; the
mobile hero is 15.4 KB. All are within the stated budgets.

Fresh mobile Lighthouse: performance 92, accessibility 100, best practices
100, SEO 100; LCP 1.7 s, CLS 0, TBT 320 ms, total transfer 177 KiB. Lighthouse
does not produce a lab INP value.

Evidence screenshots and the URL verifier output are in
`.factory/verification-evidence/live-verification-10-desktop-demo.png`,
`.factory/verification-evidence/live-verification-10-mobile-demo.png`, and
`.factory/verification-evidence/verify-url-10/`. The URL verifier reported
HTTP 200, 624 ms load, a title, `lang=en`, one H1, one main landmark, no missing
image alt text, no unlabeled buttons, and no console errors.

## Scope and missed-leverage review

No infrastructure, production data, secrets, DNS, or unrelated resources were
read or changed. The product already includes CSV import/export, the useful
adjacent capability implied by the brief. Generative AI would not improve the
deterministic, atomic stock-hold job, so its absence is not a finding.
