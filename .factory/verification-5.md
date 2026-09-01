# Stock Promise — independent verification 5

**Disposition: FAIL**

- Candidate: `42d1acecc06636936823c34e7b25b7906c8b7a91`
- URL: <https://inventory-promise-hold.sociobot.in>
- Checked: 2026-09-01 UTC
- Work order: `inventory-promise-hold-verify-5`

The candidate and live deployment match, the documented build and test gates
pass, and the main stock-hold workflow works. Release remains blocked by the
demo storage boundary, incomplete claim coverage, and keyboard focus handling.

## Release-blocking findings

### QA5-01 — High — the demo reads and writes the real browser namespace

Confirming the demo boundary showed that `/demo` is not isolated from the
browser state used by the real workspace:

- A fresh demo hold with operator `Demo-only operator` wrote
  `stock-promise:operator` to `localStorage`. The demo state itself correctly
  used `demo:stock-promise:state` in `sessionStorage`.
- Selecting **Reset demo** cleared the demo session record but left the
  non-demo operator key in persistent browser storage.
- Preloading `stock-promise:operator` with `Real workspace operator` before
  opening `/demo` caused that value to appear in the demo hold form.
- Preloading a cached license fixture caused `/demo` to prepare a verification
  request containing that non-demo license. The request was intercepted and
  answered locally by Playwright; no shared billing request was sent.

This conflicts with the persistent **Demo — sample data, nothing is saved**
notice and the requirement that demo mode neither read nor write the real
storage namespace. Evidence:

- `.factory/qa-artifacts/live-demo-storage.log`
- `.factory/qa-artifacts/live-demo-reset-storage.log`
- `.factory/qa-artifacts/live-demo-boundary.log`
- `.factory/qa-artifacts/live-demo-license-boundary.log`

### QA5-02 — Major — dialog and history navigation lose keyboard focus

Confirming keyboard-only operation found two focus-continuity failures:

- Tabbing to **Create hold** and pressing Enter opens the dialog with focus
  inside it. Pressing Escape closes the dialog, but focus moves to the document
  body instead of returning to **Create hold**.
- Selecting **Privacy** correctly focuses its heading. Browser Back and Forward
  then leave focus on the document body. Back also leaves the prior
  `Opened Privacy for Stock Promise.` announcement on the home page; Forward
  provides no route announcement.

The hold form can otherwise be completed entirely by keyboard, and the dialog
has no keyboard trap. The missing return and history focus still fail the
required dialog and route focus-management checks. Evidence:

- `.factory/qa-artifacts/live-keyboard.log`
- `.factory/qa-artifacts/live-keyboard-complete.log`
- `.factory/qa-artifacts/live-history-focus.log`

### QA5-03 — Major — visitor-facing promises are absent from the claims manifest

Confirmed that each of the 14 manifest entries has exactly one matching
`@claim:<id>` test and that every listed command passes. The cross-check also
found broader promises without their own manifest entry and tagged sandbox
test:

- The live demo notice says that nothing is saved. The `demo-isolated` claim
  only checks for live API writes and does not check browser storage or reads.
  The broader statement is also false in QA5-01.
- The README says the demo starts with three realistic SKUs and a live hold and
  can be reset from its banner. These observable promises have no corresponding
  claim entry.
- The README says hosted access uses Sociobot Microsoft Entra External ID.
  This was confirmed live, but it has no corresponding claim entry.
- The privacy page says there is no advertising or behavioral analytics and
  describes which values stay in the browser. These privacy promises have no
  tagged request/storage test.

The claims contract makes unlisted product promises release-blocking even when
an independent check happens to confirm some of them.

## First-read and demo check

**PASS.** A cold visit answers all three required questions on the first
screen:

- What it does: creates timed holds so scarce stock is not promised twice.
- Who it serves: distributors and resellers taking orders in parallel.
- What to select first: **Try it with sample data**.

One click opens a seeded Promise desk with three stock items, an active hold,
the persistent demo notice, **Reset demo**, and **Start for real**. The action
is visible in a 390×844 viewport. Screenshots:

- `.factory/qa-artifacts/first-read-desktop.png`
- `.factory/qa-artifacts/first-read-mobile.png`
- `.factory/qa-artifacts/live-workflow-final.png`

## Claims checks

Before installing dependencies, the browser claim commands could not start
because `vite` was not present in the clean clone. After the required `npm ci`,
every exact command from `.factory/claims.json` passed:

| Claim | Result |
| --- | --- |
| `demo-isolated` | PASS |
| `csv-export` | PASS |
| `offline-demo` | PASS |
| `role-boundary` | PASS |
| `rate-limit` | PASS |
| `retention-redaction` | PASS |
| `automatic-expiry` | PASS |
| `contested-stock-protection` | PASS |
| `append-only-audit` | PASS |
| `location-erasure` | PASS |
| `pro-profiles-reminders` | PASS |
| `pro-license-restore` | PASS |
| `pro-checkout-status` | PASS |
| `core-features-no-pro` | PASS |

Evidence: `.factory/qa-artifacts/claims-after-install.log`.

## Clean-checkout build and test checks

Confirmed from the candidate checkout:

- `npm ci` completed with 141 packages and 0 reported vulnerabilities.
- `npm test` passed: 3 Vitest tests, 7 Node contract tests, and 17 Rust tests.
- `npm run check` passed with 0 Svelte/TypeScript findings and Clippy warnings
  denied.
- `cargo fmt --all -- --check`, deployment-script syntax, and
  `git diff --check` passed.
- `npm run build` produced `dist/`.
- `npm run test:e2e` passed 14/14 checks.
- `BUILD_SHA=42d1acecc06636936823c34e7b25b7906c8b7a91 cargo build --release --locked`
  passed.

No container engine is installed in the verifier image. The repository's
Docker contract tests passed, and the exact locked frontend and backend
production builds completed. Evidence:
`.factory/qa-artifacts/local-gates.log`.

## Product workflow and backend checks

Confirmed in the live 390px demo:

- Quantity `0`, quantity above availability, and a blank customer are rejected.
- A 9-of-9 hold with the 480-minute maximum succeeds.
- Release restores all nine available units.
- Conversion records an outcome.
- CSV export correctly quotes commas and quote characters.
- Reset restores the shipped sample data.
- The normal workflow reports no console or page errors.

Confirmed against a fresh local release backend and temporary SQLite file:

- Initial setup, inventory creation, and supervisor session work.
- Negative stock, duplicate SKU, quantity `0`, 4-minute and 481-minute holds,
  and blank customer input return the expected client errors.
- Two simultaneous 2-unit holds against 3 units produce one `201` and one
  `409`.
- Stock cannot be reduced below an active hold.
- Release restores availability; a 3-of-3, 480-minute hold converts to
  `0` on hand, `0` held, and `0` available.
- Audit events and CSV values are retained.
- Restarting the binary with the same SQLite file retains the location,
  inventory, and both outcomes. The startup log reports generated identity on
  first start and existing identity after restart.

Evidence: `.factory/qa-artifacts/live-workflow.log` and
`.factory/qa-artifacts/local-backend-e2e.log`.

## Live identity, access, request allowance, and response policy

Confirmed that `/health` returns the full candidate SHA. The live HTML,
initial JavaScript, CSS, mobile hero, mark, service worker, and 404 document
match the candidate build byte-for-byte by SHA-256.

Confirmed live request allowances from one forwarded client identity:

- Read requests 1–80 returned `200`; request 81 returned `429` with
  `Retry-After: 59`.
- Write requests 1–20 reached authorization and returned `401`; request 21
  returned `429` with `Retry-After: 59`.
- 100 concurrent health requests returned 100 × `200` with the candidate SHA,
  at about 284 requests/second in this network sample.

Confirmed that anonymous bootstrap and hold creation are denied. A non-product
Origin receives no `Access-Control-Allow-Origin` response. Selecting live
access redirects to `sociobotcustomers.ciamlogin.com` with the product callback
URL. No customer credential was available, so authenticated live data was not
changed.

Confirmed response policy:

- HTML and legal routes: `no-cache, must-revalidate`.
- Hashed application assets: one-year immutable caching.
- API and health: `no-store`.
- Service worker: `no-cache, no-store, must-revalidate`.
- HSTS, `nosniff`, same-origin referrer policy, restrictive permissions policy,
  and header-delivered CSP with `frame-ancestors 'none'` are present.

Evidence:

- `.factory/qa-artifacts/live-identity-hashes.log`
- `.factory/qa-artifacts/live-rate-concurrency.log`
- `.factory/qa-artifacts/live-rate-header.log`
- `.factory/qa-artifacts/live-write-rate-header.log`
- `.factory/qa-artifacts/live-signin.log`
- `.factory/qa-artifacts/live-headers-cache.log`

## Accessibility, privacy, PWA, links, and performance

Confirmed on 1440px desktop and 390px mobile:

- `lang=en`, one `h1`, one `main`, image alternatives, and correct route titles
  and canonicals.
- Zero serious or critical Axe findings on home, demo, privacy, and terms.
- A visible 3px focus outline and skip link first in the tab order.
- All visible mobile controls at least 44×44 CSS px.
- No horizontal overflow at normal size or 200% root text.
- Reduced-motion preference active.
- No console, page, or request errors in the normal home/demo flow.
- Home and demo requests stayed same-origin; the demo workflow made no API
  write and set no cookie. QA5-01 records the separate browser-storage failure.
- Service worker control and update succeeded; `/demo` reloaded with its sample
  desk while offline.
- All rendered same-origin links returned `200`; the designed unknown route
  returned `404` with a route-specific title and return link.
- `robots.txt`, `sitemap.xml`, manifest, touch icon, and social image returned
  `200`.

Live Lighthouse 13 mobile:

- Performance 98
- Accessibility 100
- Best Practices 100
- SEO 100
- FCP 1.5 s; LCP 1.7 s; CLS 0; TBT 110 ms
- 171 KiB transferred

Build budgets: initial application JavaScript 93,500 bytes raw / 33.36 KiB
gzip; CSS 21,056 bytes raw / 5.57 KiB gzip; mobile hero 15,414 bytes; no web
font download. Evidence is in `.factory/qa-artifacts/`.

## Final decision

**FAIL.** Keep demo-only values in a dedicated demo namespace, prevent `/demo`
from reading real operator/profile/license preferences, make reset clear all
demo state, restore focus after dialogs and browser-history route changes, and
list/test every visitor-facing promise before repeating verification.

No deployment, infrastructure, database, key-vault, or unrelated service was
read, modified, or restarted during this verification.
