# Stock Promise — independent verification 3

Work order: `inventory-promise-hold-verify-3`

Candidate: `fb0f1d3f582807c357342488794a1a53e453c93a`

Live URL: <https://inventory-promise-hold.sociobot.in>
Verified: 2026-08-30 UTC

## Verdict: FAIL

The candidate is not releasable. The repository's own tests pass and the
single-process implementation correctly serializes contested holds, but the
mandatory claim/demo gates are absent and the live service still exposes
multiple independent SQLite states behind one URL. The deployed state is both
inconsistent and non-durable. Mandatory rate limiting is also absent from every
endpoint except PIN login.

No product source was changed. This report and the handoff are the only
repository changes made by verification.

## Release-blocking findings

### QA3-01 — Critical — the required claim contract is missing

`.factory/claims.json` does not exist. This was the first command run from the
clean candidate checkout, before installation or any other product test. There
were consequently no claim tests to execute through the demo entry point. The
work order explicitly makes a missing file release-blocking.

The omission is substantive, not clerical. The live UI and README claim atomic
holds, automatic expiry, CSV import/export, keyboard/mobile support, local data
handling, and paid reminders/profiles. None is registered under
`.factory/claims.json` with exactly one `@claim:<id>` sandbox test.

### QA3-02 — Critical — there is no one-click isolated demo

The cold first screen has no **Try it with sample data** action. `/demo`,
`?demo=1`, `.factory/demo.md`, a demo banner, reset, and an isolated demo data
namespace are all absent.

Cold-page transcription:

> Promise what’s there. Once.
>
> Create a visible, timed claim while the order is still being written.
>
> First shift setup — Name this stockroom
>
> Location name / Supervisor PIN / Open the promise desk

This describes a timed claim, but does not name the intended small distributor
or reseller, and the only action requires creating real server state. It fails
the mandatory five-second test: what it does, who it is for, and a one-click
sample action are not all present in plain words. Once an installation is
initialized, the first screen is instead a supervisor PIN gate and still has no
demo.

### QA3-03 — Critical — live traffic is split across independent SQLite states

Fresh public evidence reproduces the exact class of failure from verification
2 despite the intervening repair report:

- The prior handoff records a durable live location named `QA candidate
  e826a45`, nine inventory rows, and eight outcomes after a controlled restart.
  This verification opened the same URL cold and received
  `setup_required:true`; that previously verified state was gone from the
  responding backend.
- After QA initialized one empty state as `QA verify-3 fb0f1d3`, 120 concurrent
  `GET /api/status` requests returned 79 `setup_required:true` and 41
  `setup_required:false`. A repeat returned 80 true and 40 false.
- A valid session token produced 18 successful `GET /api/bootstrap` responses
  and 42 `401` responses in a 60-request sample. Successful responses named the
  QA location above.
- A mobile authenticated view initially loaded and passed axe, then its normal
  15-second refresh lost authorization after reaching another state; the
  expected settings control disappeared.
- All 30 identity samples and all 1,000 load-smoke responses reported
  `fb0f1d3f582807c357342488794a1a53e453c93a`. This is not mixed application
  code; it is divergent state among instances of the same candidate.

Impact: two operators can see different inventory, holds, sessions, and audit
records. A hold on one state cannot prevent a duplicate promise on another.
This defeats the product's sole safety job. The public unauthenticated setup
endpoint also lets any visitor claim an uninitialized state with a PIN.

Required repair: deploy exactly one replica with one durable `/data` mount,
prove the existing database survives a new release without loss, and repeat
the mixed-state and authenticated-session samples. Do not rely only on a
repository topology script; verify the public behavior after deployment.

### QA3-04 — High — mandatory rate limiting is absent on most endpoints

The only implemented limiter wraps `POST /api/session`. Live behavior:

- Login allowance: 10 attempts per client per 60 seconds. After one earlier
  attempt in the window, nine invalid attempts returned `401`, then the next
  two returned `429` with `Retry-After: 60`.
- `GET /api/status`: 250 concurrent requests all returned `200`; no `429` or
  `Retry-After` was observed.
- `POST /api/holds`: 60 concurrent authenticated invalid writes all reached
  validation and returned `400`; no `429` or `Retry-After` was observed.

There is no global API rate-limit middleware and no limiter on setup,
bootstrap, inventory, holds, resolve, audit, export, or logout. This fails the
backend contract requiring every server endpoint other than health to return
`429` with `Retry-After` after its allowance.

### QA3-05 — High — the required staff/supervisor boundary is not implemented

The brief says staff create holds while a supervisor converts, releases, or
expires them. In this product, the same shared supervisor PIN is required to
read the desk and create a hold, and the resulting token also permits stock
edits, conversion, release, audit, and export. Anyone able to perform the staff
job has every supervisor power. There are no staff identities or roles.

This is also the product's only sign-in mechanism. The source contains no
Microsoft Entra External ID integration and no reference to the required
`sociobotcustomers.ciamlogin.com` authority.

### QA3-06 — High — hosted privacy statements are inaccurate

The live privacy page says operational data stays in "your SQLite database on
the server you control" and that the operator controls retention and backups.
At this hosted URL, visitors do not control the server, database, backups, or
retention, and there is no deletion control. The first screen's statement that
data stays in "this installation's local database" does not explain that
customer references, names, and notes are sent to and retained by the hosted
server. The demonstrated loss/split of that server data makes these assurances
especially unsafe.

### QA3-07 — High — Dockerfile violates the mandatory Rust image contract

The backend stage is `FROM rust:1.98-bookworm`. The supplied backend contract
requires `rust:1-slim` or `rust:1-alpine` and explicitly forbids pinning a Rust
minor. The local locked release build passed, but the checked-in Dockerfile is
not compliant with the factory build contract. No container engine was
available in this verifier image, so an end-to-end local image build was not
possible.

## Additional acceptance findings

### QA3-08 — Medium — required site structure and metadata are incomplete

- `/privacy` and `/terms` retain the home title, `Stock Promise — live
  inventory holds`, instead of route-specific titles.
- `/does-not-exist` returns HTTP `404` but renders the normal setup/access
  application, not a designed 404 page with a way back.
- `robots.txt` and `sitemap.xml` return `404` HTML.
- Canonical, Open Graph title/image, Twitter card, and apple-touch icon metadata
  are absent.
- The landing page has no how-it-works, non-goals/privacy, or paid-tier section;
  price is visible only after entering the supervisor area.
- The footer omits “Built by Param Factory” and a build/version identifier.

### QA3-09 — Medium — copy/process artifacts do not meet the plain-words contract

`.factory/copy-audit.md` is absent. The first headline, “Promise what’s there.
Once.”, does not identify the hold job or intended user, and footer copy such as
“Soft holds, clearly seen.” is decorative rather than actionable. The
researched brief says subscription; the product instead advertises a `$39
one-time` license without documenting the deviation.

## Clean-checkout and build evidence

The tree began clean at the exact requested candidate. Environment: Node
`22.23.2`, npm `10.9.8`, rustc/cargo `1.98.0`, Playwright `1.58.2`.

- `npm ci` — passed; 139 packages installed; audit reported 0 vulnerabilities.
- `npm test` — passed: 1 Vitest, 6 Node contract tests, and 9 Rust tests.
- `npm run check` — passed: 0 Svelte/TypeScript diagnostics; Clippy passed with
  warnings denied.
- `cargo fmt --all -- --check` — passed.
- `bash -n deploy/*.sh` — passed.
- `npm run build` — passed and produced `dist/`.
- `BUILD_SHA=fb0f1d3f... cargo build --release --locked` — passed.
- `npm run test:e2e` — 5/5 passed against a fresh local database.

Production bundle:

- JavaScript: 75,665 bytes raw / 28.01 KB gzip (under 200 KB).
- CSS: 18,457 bytes raw / 5.05 KB gzip (under 50 KB).
- Mobile image: 15,414 bytes (under 300 KB).
- No downloaded web fonts.

The locally built JS and CSS SHA-256 hashes exactly matched the corresponding
live assets. `/health` reports the exact candidate SHA, so the live frontend
and backend identity both match the candidate.

## Functional and API evidence

Against a clean local database, the checked-in E2E suite covered setup, empty
state, stock creation, hold creation/conversion, CSV download, legal routes,
anonymous denial, invalid-PIN recovery, keyboard navigation, 390 px layout,
axe, reduced motion, cache/security headers, service-worker update, and offline
reload.

A separate local release-binary test created a location and item, stopped the
process, restarted it on the same SQLite path, logged in again, and recovered
the same location and `PERSIST-1` item with seven units.

Fresh live API checks on the one initialized state passed:

- Bad PIN, invalid SKU, blank item, negative/over-maximum stock, zero and
  over-maximum hold quantity, 4/481-minute duration, blank customer, missing
  inventory, duplicate SKU, stock reduction below an active hold, and repeated
  resolution returned the expected `400`, `404`, or `409`, followed by
  successful recovery.
- Stock boundaries `0` and `100,000,000` were accepted.
- Two simultaneous two-unit holds against three available units returned
  exactly `201` and `409`; availability never went negative.
- A released hold restored availability. A three-of-three hold at the
  480-minute duration boundary converted successfully and left
  on-hand/held/available at `0/0/0`.
- The append-only audit view contained setup, inventory, hold creation,
  release, and conversion events.
- CSV returned `text/csv`, the attachment filename, one row per hold, and
  correctly escaped a comma and quote in the note.
- Anonymous bootstrap, hold, audit, and export requests returned `401`.

The live QA run created only clearly labelled test state on one previously
empty backend: location `QA verify-3 fb0f1d3`, SKUs `QA3-RACE`, `QA3-ZERO`, and
`QA3-MAX`, two resolved holds, and no active hold. Nothing was deleted.

## Browser, accessibility, privacy, PWA, and performance evidence

- Desktop 1440×1000 and mobile 390×844 had no horizontal overflow.
- Axe found zero serious/critical issues on the cold/access and authenticated
  views that loaded successfully.
- The skip link was first in keyboard order and had a visible solid outline.
  Authenticated dialog focus entered the modal and Escape closed it.
- All visible sampled controls were at least 44×44 CSS px.
- `prefers-reduced-motion: reduce` was active and respected by the stylesheet.
- Successful online flows had zero console errors, page errors, or failed
  requests. Requests remained same-origin and the browser stored no cookies.
- An attacker-origin request received no `Access-Control-Allow-Origin` header.
- The service worker installed, controlled the page after reload, updated from
  `/sw.js`, and served the shell offline with an explicit recovery screen. The
  expected offline API failure did appear as a browser
  `net::ERR_INTERNET_DISCONNECTED` console message.
- Security headers included CSP with header-only `frame-ancestors 'none'`,
  HSTS, nosniff, same-origin referrer policy, and restrictive permissions.
- Cache policy: HTML/legal `no-cache, must-revalidate`; API/health `no-store`;
  hashed JS/CSS one-year immutable; service worker no-store; stable image one
  day.
- Lighthouse 13 mobile: Performance 100, Accessibility 100, Best Practices
  100, SEO 100; FCP 1.4 s, LCP 1.5 s, TBT 0 ms, CLS 0.001, 109 KiB transferred,
  7 requests.
- A 1,000-request health smoke completed with 1,000 HTTP 200 responses and no
  transport errors at 1,114.82 requests/s; p50 29.3 ms, p95 159.9 ms, p99
  207.8 ms, max 253.4 ms. Every response reported the candidate SHA.

## Final disposition

**FAIL.** Do not release this candidate. Fix the claim/demo gates, deploy one
durable SQLite state, add endpoint-wide rate limiting, implement the required
staff/supervisor access boundary (and required identity provider if sign-in is
retained), correct hosted privacy copy/control, and then repeat verification
from a genuinely clean demo path.

Library/CLI consumer packaging is not applicable to this web-with-backend
artifact. No prohibited shared service, database, key vault, app setting, DNS,
billing resource, or infrastructure control plane was read or changed.
