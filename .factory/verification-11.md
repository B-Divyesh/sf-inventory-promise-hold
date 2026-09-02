# Independent verification 11 — FAIL

Verified candidate: `f0f6b4d909889720794e162a49ea5da067a7df91`

Live URL: <https://inventory-promise-hold.sociobot.in>

Verification date: 2026-09-02 UTC

## Verdict

**FAIL.** Product behavior and the deployed service pass the acceptance checks,
but the candidate does not pass its mandatory gates from a clean clone. A
registered claim command times out during its cold backend build, and `npm
test` separately times out during its cold release build. Both pass only after
the relevant Rust profile has been compiled.

Defect count: **0 blocker, 1 high, 0 medium, 0 low**.

## Release-blocking finding

### HIGH — clean-clone test timeouts make required gates fail

After `npm ci` in the clean candidate checkout, the first exact registered
claim command was:

```text
npm run test:e2e -- --grep @claim:demo-isolated
```

The frontend built, but Playwright exited 1 with `Timed out waiting 120000ms
from config.webServer` while `cargo run --quiet` was still compiling the
backend. After that compilation completed, the exact command passed in 4.9
seconds.

The first complete `npm test` run independently failed. Its
`release binary emits the configuration record with only PORT` subtest hit its
180,000 ms test timeout while `cargo build --release --locked` was still
compiling. The Node runner finished after 314,241 ms with 9 passes and 1
cancelled test, exit 1. After the release profile was compiled, `npm test`
passed: 3 frontend tests, 10 Node tests, and 20 Rust tests.

This is not a product-runtime failure: the claim and startup behavior both pass
warm, and the release binary emits the required configuration record. It is
still release-blocking because `.factory/claims.json` says every exact claim
test must pass from the clean sandbox, and the repository definition of done
requires `npm test` to pass locally. The cold-build timeouts need to include
realistic compilation time or compile before starting their assertion clocks.

## First-read and demo gate

**PASS.** The cold live first screen says “Hold scarce stock before it is
promised twice,” names distributors and resellers taking orders in parallel,
and presents **Try it with sample data** beside “Open a sample stockroom.” One
click opens `/?demo=1` with three SKUs, one active hold, and the persistent
“Demo — sample data, nothing is saved” banner with Reset demo and Leave demo.

The first screen therefore explains what the product does, who it is for, and
what to click first in plain words. It also provides the required one-click
sample.

## Claims gate

`.factory/claims.json` exists with 22 entries. After the clean dependency
install, every listed command was invoked independently:

- **21 passed on their listed run.** These cover demo reset, no tracking,
  browser/session storage, hosted CIAM access, location authorization, first
  setup, durable storage, CSV export, offline reload, role boundaries, rate
  limiting, retention, automatic expiry, contested stock, append-only audit,
  erasure, Pro behavior, checkout status, and free core features.
- **1 failed on its listed cold run:** `demo-isolated`, because the configured
  web-server timeout expired during compilation. The exact warm rerun passed.

No material claim-like sentence on the live landing page or in `README.md` was
found without a corresponding claim entry. The live billing checkout still
returns 404, matching the tested “Paid upgrades are temporarily unavailable”
copy; the product exposes no checkout link.

## Build and automated checks

- `npm ci`: passed; 143 packages installed and npm reported 0 vulnerabilities.
- First `npm test`: **failed** after 314.2 seconds because the 180-second
  release-startup test timed out during the cold release build.
- Warm `npm test`: passed — 3 Vitest, 10 Node, and 20 Rust tests.
- `npm run check`: passed — 0 Svelte/TypeScript diagnostics; Clippy passed with
  warnings denied.
- `cargo fmt --all -- --check`: passed.
- `BUILD_SHA=f0f6b4d909889720794e162a49ea5da067a7df91 npm run build`: passed and
  produced `dist/`.
- `BUILD_SHA=f0f6b4d909889720794e162a49ea5da067a7df91 cargo build --release --locked`:
  passed.
- `npm run test:e2e:all`: passed — 21 product browser tests and 1 hosted-auth
  browser test.
- Docker and Podman are unavailable in this verification container, so image
  assembly was not rerun. Dockerfile contracts passed in the Node suite.

## Functional and backend evidence

The live demo was used only in its isolated sample namespace:

- Quantity `0` was rejected with “Value must be greater than or equal to 1.”
  Quantity `999` was rejected with “Value must be less than or equal to 9.”
- Holding all 9 available valve units changed the item to 0 available and
  disabled it as Fully held. Releasing the hold restored 9 available.
- A normal two-unit hold with customer, operator, and note was created and
  converted. CSV export contained the header and both new outcomes; Reset demo
  restored exactly three SKUs and one active sample hold.

An independent candidate-stamped release binary used a fresh temporary SQLite
database. Blank customer and quantity zero returned clear 400 responses. Two
simultaneous requests each asking for two of three units produced exactly one
201 and one 409. After a process restart, the same session could read the
location, SKU, one active hold, and one remaining available unit. `/api/audit`
returned 200, `/health` retained the exact build SHA, and the required startup
configuration record was present.

Live backend evidence:

- `/health` returned
  `f0f6b4d909889720794e162a49ea5da067a7df91` and `status:"ok"`.
- `/api/auth/config` returned `mode:"ciam"`.
- The sign-in button redirected only to
  `sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650`
  with client `25c704f4-465a-47af-80ab-2c489466b697` and this product's
  `/auth/callback` URL.
- Read allowance observed: 80 requests per client per 60 seconds; request 81
  returned 429 with `Retry-After: 59`.
- Write allowance observed: 20 requests per client per 60 seconds; request 21
  returned 429 with `Retry-After: 59`. The first 20 unauthenticated write probes
  returned 401 and did not mutate data.
- A 100-request concurrent read smoke using distinct client identities returned
  100/100 HTTP 200 in 237 ms; health remained 200 afterward.

## Privacy, accessibility, PWA, and response policy

- The whole live home-to-demo mutation/export/reset flow requested only the
  product origin, created no cookies, and produced no console errors, page
  errors, or failed requests.
- Axe found 0 serious/critical findings on home, demo, privacy, terms, and the
  designed 404 route.
- At 390 px there was no horizontal overflow, every visible control was at
  least 44 px in both dimensions, and 200% text kept document width at 390 px.
- The first keyboard focus was the skip link with a visible 3 px sand-colored
  outline. Enter activated it. Dialog focus started inside the dialog; Escape
  closed it and returned focus to the opener. Reduced-motion preference was
  active.
- The service worker controlled the demo, updated to
  `/sw.js?v=f0f6b4d909889720794e162a49ea5da067a7df91`, used a build-versioned
  cache, and reloaded the sample offline.
- The fleet URL verifier reported HTTP 200, 634 ms load, `lang=en`, one H1, one
  main landmark, no missing image alt text, no unlabeled buttons, and no console
  errors. Evidence is under `.factory/verification-evidence/verify-url-11/`.
- HTML routes use `no-cache, must-revalidate`; entry assets use one-year
  immutable caching; `/sw.js`, `/api/*`, and `/health` use `no-store` policies.
  CSP is a response header with `frame-ancestors 'none'`; HSTS, nosniff,
  same-origin referrer policy, and restrictive Permissions-Policy are present.
- All intended internal links returned 200. The deliberately unknown route
  correctly returned the designed 404 page. `robots.txt` and `sitemap.xml`
  returned 200 and the sitemap lists home, demo, privacy, and terms.

## Deployment match and performance

The live HTML, CSS, entry JavaScript, and shared JavaScript SHA-256 hashes
exactly match the candidate production build. The entry JS is 34.6 KB gzip;
the lazily available shared JS is 69.6 KB gzip; CSS is 5.9 KB gzip; and the
mobile hero image is 15.4 KB. No fonts are downloaded. These are within budget.

Fresh mobile Lighthouse results: performance **93**, accessibility **100**,
best practices **100**, SEO **100**; FCP 1.50 s, LCP 1.65 s, TBT 295 ms, CLS 0,
and total transfer 181,706 bytes. Lighthouse does not provide a lab INP value.

## Scope and missed-leverage review

No infrastructure, production data, secrets, DNS, or unrelated resources were
read or changed. Production write checks were unauthenticated boundary probes;
all functional mutations used the demo or a temporary local database. CSV
import/export already supplies the useful adjacent capability implied by the
brief. Generative AI would add risk and no value to deterministic atomic stock
holds, so its absence is not a finding.
