# Timed inventory holds — review 4

Review date: 2026-09-05 UTC  
Live URL: <https://inventory-promise-hold.sociobot.in>  
Implementation candidate: `26292139a5a935a48fcc9146b6c1dc4745868373`  
Documentation baseline: `3be6147af141b90fb2f7c76a713863fd58bb74f6`

## Verdict

**PASS**

Finding count: **0**  
Untested claim count: **0**

The live `/health` response reports the implementation candidate above. The
later documentation baseline changes only `.factory` reports and evidence;
it does not change the reviewed product.

## First screen and sample

Fresh desktop (1440 px) and phone (390 px) browser contexts both showed the
job before scrolling: **“Hold scarce stock before it is promised twice.”**
They name distributors and resellers taking orders in parallel and put **Try
it with sample data** beside **Open a sample stockroom.**

One click opened `/?demo=1`. It showed Harbor Parts, three named SKUs, the
Northline Plumbing active hold, and the persistent **Demo — sample data,
nothing is saved** label with Reset demo and Leave demo. The demo sent no API
requests during the create, convert, export, and reset flow.

I checked normal, invalid, boundary, and recovery paths in a fresh demo:

- Quantity `0` is rejected with “Value must be greater than or equal to 1.”
- A two-unit hold for a new customer was created, converted, and included in
  `stock-promise-holds.csv`.
- Holding all nine available valve units changed the control to Fully held.
  Releasing the hold restored nine available units.
- Reset restored exactly three SKUs and the Northline Plumbing active hold.

## Claims and local gates

After `npm ci`, I invoked every command declared in `.factory/claims.json`
separately from this clean checkout. All 22 passed:

| Claims | Result |
| --- | --- |
| `demo-isolated`, `demo-seed-reset`, `no-tracking`, `browser-storage` | PASS |
| `hosted-token-storage`, `hosted-access`, `location-data-access`, `first-supervisor-setup` | PASS |
| `shared-durable-storage`, `csv-export`, `offline-demo`, `role-boundary` | PASS |
| `rate-limit`, `retention-redaction`, `automatic-expiry`, `contested-stock-protection` | PASS |
| `append-only-audit`, `location-erasure`, `pro-profiles-reminders`, `pro-license-restore` | PASS |
| `pro-checkout-status`, `core-features-no-pro` | PASS |

The first cold `npm test` run completed after its release compilation, which
is now performed before the assertion timer. A warm confirmation run passed:
3 frontend tests, 11 Node contract/startup tests, and 20 Rust tests. The
following also passed: `npm run check`, `cargo fmt --all -- --check`,
`VITE_BUILD_SHA=26292139a5a935a48fcc9146b6c1dc4745868373 npm run build`, and
`npm run test:e2e:all` (21 product browser tests and one hosted-auth test).
The production build produced `dist/`.

No visitor-facing claim found on the landing page, legal pages, demo, or README
was absent from the claim manifest.

## Live service checks

- `/health` returned HTTP 200 with the full implementation SHA and `status:ok`.
- `/api/auth/config` returned CIAM mode. Anonymous live bootstrap and hold
  creation remain unauthorized; no operational data was read or changed.
- The 81st read from one forwarded client address returned HTTP 429 with
  `Retry-After: 57`. The 21st write probe from another address returned HTTP
  429 with `Retry-After: 59`. The preceding write probes were unauthorized.
- The demo service worker controlled the page as
  `/sw.js?v=26292139a5a935a48fcc9146b6c1dc4745868373` and reloaded the populated
  sample while offline after its first visit.

## Accessibility, privacy, and routes

On home, demo, privacy, terms, and the designed unknown route, each document
had its route title, `lang=en`, one H1, and one main landmark. Playwright Axe
found zero serious or critical findings on every route. Phone and desktop had
no horizontal overflow, and the page emitted no application console errors.

All discovered same-origin links returned 200. The unknown route deliberately
returned HTTP 404 with the working designed Page not found page; its browser
network message is expected for that document status, not a product defect.
Privacy, terms, robots, sitemap, demo, and legal navigation worked. The demo
request log stayed same-origin and created no API request during the tested
sample workflow.

## Earlier findings

Every finding from earlier reviews and verifications was checked again.

| Earlier finding | Current disposition |
| --- | --- |
| Review 1 F-1-1 through F-1-12 | Fixed: the sample action is a registered claim; phone facts fit; 404 has the shared shell and plain heading; demo labels, audit wording, README headings and setup wording, and terms copy are clear and consistent. |
| Review 2 F-2-1 through F-2-13 | Fixed: demo entry cannot update live license state; public screens do not claim “Shared live”; persistence and setup boundaries are claimed and tested; product, retention, Pro, README, exit, legal-link, and 404-footer wording/structure are corrected. |
| Review 3 F-3-1 through F-3-7 | Fixed: hosted token storage and location access have dedicated claims; offline is a first-screen fact; the landing contains a real sample preview; the unavailable tier is not presented as a price-less offer; jargon was removed. |
| Verification 11 high cold-build timeout | Fixed: the clean cold claim/test run compiled before readiness and assertion timing. |
| Verification 12 low mobile tab wrapping | Not reproduced. At 390 px, Inventory holds and Stock & settings wrap only at spaces, remain complete, and have 56 px tab controls with no overflow. |

## Scope

This review did not change product code, deployment resources, production
records, DNS, billing, or secrets. Functional writes were limited to isolated
sample data; the live write allowance check was unauthenticated.
