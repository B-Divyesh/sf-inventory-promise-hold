# Stock Promise — polish round 2

Polished on 2026-09-02 from review commit `23128e135fe7195f55c652c380af4fb7a95a8a00`.
Every finding in `review-1.md` and `review-2.md` is resolved. Functional live
verification ran against build `e16a5610c97e0b19b036fd8f6d41125dbb22ee5c`
at <https://inventory-promise-hold.sociobot.in>.

## Review 2 findings

| Finding | Change made | Evidence |
| --- | --- | --- |
| F-2-1 | Live license checks now have an abort signal and a current-namespace guard. Entering demo cancels the live request before any verdict write. The isolation claim now starts at `/`, holds a hostile live response open, enters `/?demo=1`, releases it, and compares every real key byte-for-byte. | `@claim:demo-isolated`; live test `one-click demo invalidates a delayed live license response`; <https://inventory-promise-hold.sociobot.in/?demo=1>. |
| F-2-2 | Removed the network-derived “Shared live” label from public and access screens. Demo retains the accurate “Sample data” label. | `@claim:demo-seed-reset` asserts no public connection label; live test `cold mobile and desktop first screens state the job and action`; [mobile first screen](qa-artifacts/polish-2-mobile-first-screen.png). |
| F-2-3 | Added the `shared-durable-storage` manifest claim. Its file-backed test writes stock plus active and resolved holds, reads them in another session, reopens SQLite, then confirms stock, both hold states, and four audit entries remain. | `cargo test claim_shared_durable_storage_survives_sessions_and_restart --quiet`; Privacy and README live URLs. |
| F-2-4 | Added the `first-supervisor-setup` claim and test. Rewrote deployment prose as required configuration and post-release instructions instead of unregistered guarantees. | `cargo test claim_first_supervisor_creates_the_location_once --quiet`; contract tests `container uses current stable Rust…`, `topology verifier…`, and `release delegates…`. |
| F-2-5 | Standardized the working area on **inventory holds**: **Open inventory holds**, **Manage sample inventory holds**, and **Inventory holds**. | Contract test `reviewed visitor copy uses one plain term for each concept`; [populated demo](qa-artifacts/polish-2-demo-desktop.png); live demo URL. |
| F-2-6 | Renamed the mixed limits/privacy section to **Limits and data retention**. | Copy contract test; home live URL. |
| F-2-7 | Renamed the paid-feature heading to **Pro profiles and reminders**. | Copy contract test; `@claim:pro-profiles-reminders`. |
| F-2-8 | Renamed the README section to **Try the sample stockroom**. | Copy contract test; README. |
| F-2-9 | Replaced “CIAM” with “shared Sociobot customer sign-in service” in local-run guidance. | Copy contract test rejects `CIAM` in README. |
| F-2-10 | Replaced the opening implementation phrase with “a record of past changes that cannot be edited.” | Copy contract test rejects “append-only audit record” in the README opening. |
| F-2-11 | Renamed the demo exit action to **Leave demo**, matching its home destination and namespace cleanup. | `@claim:demo-isolated`; live delayed-response demo test. |
| F-2-12 | Legal pages now use `<a href="/">Return home</a>` with SPA enhancement. | Browser test `legal pages stay semantic and reachable`; live test `legal routes, history focus, and 404 keep the complete shell`. |
| F-2-13 | The Vite build injects the 12-character build SHA into the static 404 footer. | Browser test `public routes keep one route-specific canonical and description`; [mobile 404](qa-artifacts/polish-2-404-mobile.png); live unknown URL returned 404 with `build e16a5610c97e`. |

## Review 1 regression map

| Finding | Current state | Evidence |
| --- | --- | --- |
| F-1-1 | “Open a sample stockroom” remains registered under `demo-seed-reset`. | `@claim:demo-seed-reset`. |
| F-1-2 | The action and three facts remain inside the 390×844 first view. | Browser test `390px first screen contains…`; [mobile first screen](qa-artifacts/polish-2-mobile-first-screen.png). |
| F-1-3 | Static 404 retains the shared header, Demo/Privacy navigation, full footer, icons, and original art. | Browser route test; [mobile 404](qa-artifacts/polish-2-404-mobile.png). |
| F-1-4 | Static and SPA 404 headings remain **Page not found**. | Contract copy test and route browser test. |
| F-1-5 | Demo status remains **Sample data**. | `@claim:demo-seed-reset`; live demo test. |
| F-1-6 | Demo still has no supervisor lock control. | `@claim:demo-seed-reset`. |
| F-1-7 | Visitor copy consistently uses **audit record**. | Copy contract test and `@claim:append-only-audit`. |
| F-1-8 | README heading remains **Timed inventory holds for parallel orders**. | Copy contract test. |
| F-1-9 | README still says sign-in roles set what each person can do. | Copy contract test. |
| F-1-10 | README still names the callback as the sign-in return address. | Copy contract test. |
| F-1-11 | Deployment wording now gives explicit `/data` and one-replica requirements. | Contract topology tests; live revision `0000033` reported one mounted `/data` volume and min/max replicas of 1. |
| F-1-12 | Terms retains the plain fair-use sentence. | Copy contract test; <https://inventory-promise-hold.sociobot.in/terms>. |

## Verification evidence

- Fresh clone `/tmp/stock-promise-claims-WBWd1g`: `npm ci`, then all 20
  `.factory/claims.json` commands independently — pass.
- Same clean clone: `npm test` (3 Vitest, 9 Node contract, 19 Rust),
  `npm run check`, `cargo fmt --all -- --check`, `npm run build`, and
  `npm run test:e2e` (20/20) — pass.
- Initial JavaScript: 95.43 KB raw / 34.06 KB gzip. CSS: 21.29 KB raw /
  5.61 KB gzip.
- Live polish suite: 4/4, including mobile/desktop cold reads, delayed license
  response, one-click demo/reset, route focus, legal semantics, static 404,
  headers, rate limiting, offline reload, and Axe serious/critical checks.
- Worker URL verifier: 200 in 617 ms, title/lang/one H1/main/alts present, no
  unlabeled buttons, and no console errors. Evidence:
  `qa-artifacts/polish-2-verify-url/`.
- Lighthouse mobile: Performance 99, Accessibility 100, Best Practices 100,
  SEO 100; FCP 1.5 s, LCP 1.6 s, TBT 40 ms, CLS 0, 174 KiB transferred.
  Full report: `qa-artifacts/polish-2-lighthouse.json`.

No review finding remains open.
