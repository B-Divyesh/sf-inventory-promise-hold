# Independent verification 6 — PASS

- Candidate commit: `4abf5cdb2918d114564c2ccc780c6aa2633c0ac8`
- Live URL: <https://inventory-promise-hold.sociobot.in>
- Verified: 2026-09-01 UTC
- Result: **PASS**

## First read

Cold-opening the live home page answered the three required questions in plain
words. It says that Stock Promise holds scarce stock before it is promised
twice, names distributors and resellers taking parallel orders, and presents
**Try it with sample data** as the first primary action with the explanation
“See a working stockroom immediately.” The click opened the isolated demo.

## Registered claims

`.factory/claims.json` is present. Every listed command passed from this clean
checkout: 18 of 18 claim checks.

- Browser checks passed for demo isolation, seed/reset, no tracking, documented
  browser storage, CSV export, offline demo, Pro profiles/reminders, license
  restore, unavailable checkout status, and free core features.
- Rust checks passed for hosted CIAM access, role boundaries, rate allowance,
  retention redaction, automatic expiry, contested-stock protection,
  append-only audit rows, and location erasure.

## Repository checks

- `npm ci`: passed; 0 package vulnerabilities reported.
- `npm test`: passed — 3 Vitest, 7 Node contract, and 17 Rust tests.
- `npm run check`: passed — Svelte check reported 0 errors and 0 warnings;
  Clippy passed with warnings denied.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- `npm run build`: passed and produced `dist/`.
- `npm run test:e2e`: passed — 19 Playwright scenarios.
- `BUILD_SHA=4abf5cdb2918d114564c2ccc780c6aa2633c0ac8 cargo build --release --locked`:
  passed.

The produced initial JavaScript is 94,817 bytes raw / 33,931 bytes gzip and
CSS is 21,056 bytes raw / 5,566 bytes gzip. These are within the applicable
initial-JS and CSS budgets.

## Live product checks

- `/health` returned HTTP 200, `status: ok`, and the exact candidate SHA.
  One hundred concurrent live health requests all returned 200 and that same
  SHA.
- The live API returned `{"mode":"ciam"}` from `/api/auth/config`; the claim
  regression also confirms the Sociobot Microsoft Entra External ID defaults.
- In the sample at desktop and 390 px mobile: three sample SKUs and one active
  hold appeared; an over-available quantity was rejected by native validation
  (“Value must be less than or equal to 9.”); a valid quantity then created a
  sample hold; Reset restored three SKUs and one hold and cleared demo keys.
- The persistent sample-data banner was present. The desktop keyboard check
  focused the skip link first and moved to `#main`. Mobile had no horizontal
  overflow at 200% text and no visible control smaller than 44 px.
- `prefers-reduced-motion: reduce` was active in the 390 px check. Axe reported
  zero serious or critical findings on live home and demo. No console or page
  errors occurred in desktop or mobile flows.
- The demo request log contained only same-origin product resources. It set no
  cookies and made no API request while creating and resetting sample data.
- After a first visit, `/demo` reloaded offline. The active service-worker URL
  and cache name contained the full candidate SHA.
- All discovered internal landing-page links returned 200. `/privacy`,
  `/terms`, `/demo`, robots, sitemap, and the designed 404 route responded as
  expected.
- Response headers included CSP with `frame-ancestors 'none'`, HSTS,
  `nosniff`, same-origin referrer policy, and a restrictive permissions policy.
  HTML routes were revalidated; the hashed JavaScript was immutable; the
  service worker was not cached as a long-lived asset; API and health responses
  used `no-store`.
- From one fresh forwarded client address, 80 `/api/status` requests returned
  200; request 81 returned `429` with `Retry-After: 59`. The observed public
  read allowance is 80 requests per 60 seconds. A separate client address
  received 200 afterward.

## Backend durability check

Using only a temporary local SQLite file, a release binary started with the
candidate SHA, accepted a temporary local setup, and returned `setup_required:
false` after a clean restart using the same file. One hundred concurrent local
health requests returned 200 with the same SHA. The temporary check did not
touch the deployed data directory.

## Defects

No release-blocking, high, medium, or low severity defects found.
