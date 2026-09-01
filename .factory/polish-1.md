# Stock Promise — polish round 1

Date: 2026-09-01 UTC  
Reviewed candidate: `4abf5cdb2918d114564c2ccc780c6aa2633c0ac8`  
Review report: `.factory/review-1.md`  
Product repair commit: `b4fe5c74d419d83b35fe39514770a5ca3ecfc586`  
Live URL: <https://inventory-promise-hold.sociobot.in>

## Review 1 finding map

| Finding | Change made | Evidence |
| --- | --- | --- |
| F-1-1 | Replaced “See a working stockroom immediately” with “Open a sample stockroom.” Registered the landing promise under `demo-seed-reset`; its one tagged test now starts at home and enters `/?demo=1` in one click. | `@claim:demo-seed-reset`; contract test `every registered claim has one tagged sandbox test`; live test `cold mobile first screen and one-click query demo`; <https://inventory-promise-hold.sociobot.in/?demo=1>. |
| F-1-2 | Mobile landing order now puts copy, action, all three facts, and the live action before the environmental image, with tighter phone spacing. | Browser test `390px first screen contains the job, action, and three plain facts` asserts the facts end at or above 844 px; [live mobile screenshot](qa-artifacts/polish-1-live-mobile-first-screen.png). |
| F-1-3 | Rebuilt static `404.html` with the Stock Promise wordmark, Demo/Privacy navigation, legal footer, favicon, apple-touch icon, social metadata, and original stockroom art. | Browser test `public routes keep one route-specific canonical and description`; live test `route metadata, history focus, legal copy, and static 404 shell`; [live 404 screenshot](qa-artifacts/polish-1-live-404-mobile.png); <https://inventory-promise-hold.sociobot.in/not-a-real-route> returns 404. |
| F-1-4 | Replaced both static and SPA headings with “Page not found”. | Contract test `reviewed visitor copy uses one plain term for each concept`; route browser tests; live 404 URL above. |
| F-1-5 | Demo header status now says “Sample data” and never renders the live connection indicator. | `@claim:demo-seed-reset`; live test `desktop demo labels isolated sample state`; [live desktop demo screenshot](qa-artifacts/polish-1-live-demo-desktop.png). |
| F-1-6 | Removed the non-functional supervisor lock control from demo mode; live workspaces retain the real sign-out/lock control. | `@claim:demo-seed-reset` asserts no Lock supervisor button; live desktop demo test and screenshot above. |
| F-1-7 | Standardized the UI, privacy page, README, and code comments on **audit record**. Defined it as: “The audit record keeps past changes and cannot be edited.” | Contract test `reviewed visitor copy uses one plain term for each concept`; `@claim:append-only-audit`; live privacy copy assertion. |
| F-1-8 | README heading is now “Timed inventory holds for parallel orders”. | Contract test `reviewed visitor copy uses one plain term for each concept`. |
| F-1-9 | README now says “Sign-in roles set what each person can do”. | Same copy contract test. |
| F-1-10 | README now calls the callback URL the “sign-in return address”. | Same copy contract test. |
| F-1-11 | README now says deployment keeps SQLite in `/data` and runs one app replica. | Same copy contract test; deployment contract tests; live revision `sf-inventory-promise-hold--0000031` reported one `/data` mount and min/max replicas of 1. |
| F-1-12 | Terms now says: “Do not interfere with normal service use or present inaccurate stock availability to customers.” | Same copy contract test; live legal-copy assertion; <https://inventory-promise-hold.sociobot.in/terms>. |

## Cumulative regression coverage

Earlier verification findings remain fixed:

- Original persistence, access, identity, login-rate, legal-route, cache,
  mobile-target, and security-header defects: `npm test`, the release topology
  check, live identity/header test, and the 20-scenario browser suite pass.
- QA-01 and QA3-03: revision `0000031` has one replica and one Azure Files
  mount at `/data`; `/health` reports the repair SHA.
- QA3-01 through QA3-09: 18 claims exist with exactly one tagged test each;
  query and path demos are seeded and isolated; rate, role, hosted access,
  Docker, metadata, 404, catalog, and copy contracts pass.
- QA4-01 through QA4-05: unavailable checkout stays absent; all material
  promises are registered; 44 px/200% reflow, canonical metadata, and the real
  footer build identifier pass.
- QA5-01 through QA5-03: hostile live browser state remains untouched by demo
  use; Reset removes all demo keys; dialog/history focus and the expanded
  privacy/demo/access claims pass.

## Verification evidence

- Fresh clone at `b4fe5c74d419d83b35fe39514770a5ca3ecfc586`:
  all 18 manifest commands passed independently.
- `npm test`: 3 Vitest, 9 Node contract, and 17 Rust tests passed.
- `npm run check`: zero Svelte/TypeScript findings; Clippy passed with warnings
  denied. `cargo fmt --all -- --check` and deployment shell syntax passed.
- `npm run test:e2e`: 20/20 scenarios passed, including axe on home, demo,
  legal pages, and static 404.
- Locked release build passed. Initial app JS is 95.02 KB raw / 33.98 KB
  gzip; CSS is 21.29 KB raw / 5.61 KB gzip.
- Live Playwright repair suite: 5/5 passed, including cold mobile, query demo,
  history focus, 404 shell, headers/rate allowance, and offline reload.
- `npx @axe-core/cli`: 0 violations on home, query demo, privacy, and terms.
- Worker URL verifier: HTTP 200, 593 ms, correct title/lang/H1/main/alts,
  no unlabeled buttons, and no console errors. Evidence is in
  `qa-artifacts/polish-1-verify-url/`.
- Lighthouse mobile: Performance 99, Accessibility 100, Best Practices 100,
  SEO 100; FCP 1.50 s, LCP 1.65 s, TBT 98 ms, CLS 0, 177,362 transferred
  bytes. Full JSON: `qa-artifacts/polish-1-lighthouse.json`.

No finding from review 1 or the earlier verification history remains open.
