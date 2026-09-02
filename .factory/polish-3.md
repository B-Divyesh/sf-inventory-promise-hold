# Stock Promise — polish round 3

Polished on 2026-09-02 from review commit `342539596fa0490e5fa63ce9b3d8fabf3fbe2782`.
The product repair is `ae65df265dc1fd7b14b44d7b081033cadb2b40ff`, deployed at
<https://inventory-promise-hold.sociobot.in>. This document maps every finding
in the three adversarial reviews. All paths below were checked after the final
deployment, whose `/health` response reports that repair SHA.

## Round 3 findings

| Finding | Change made | Evidence |
| --- | --- | --- |
| F-3-1 | Split hosted sign-in storage into `hosted-token-storage`. The hosted fixture runs the actual MSAL cache hydration path with the production CIAM configuration, then proves its token cache is in `sessionStorage`, absent from `localStorage`, and sent as a Bearer token. The fixture is enabled only by the test build flag. | `npm run test:e2e:hosted -- --grep @claim:hosted-token-storage`; `.factory/claims.json`; live Privacy check in `polish-3-live.spec.ts`; <https://inventory-promise-hold.sociobot.in/privacy>. |
| F-3-2 | Registered `location-data-access` and added a CIAM-signed, SQLite-backed contract test. It rejects unauthenticated, wrong-tenant, and role-less callers from bootstrap and audit data while confirming the staff boundary. The access screen now states the concrete sign-in action. | `cargo test claim_location_data_access_rejects_unauthenticated_wrong_tenant_and_roleless_requests --quiet`; live route check in `polish-3-live.spec.ts`; <https://inventory-promise-hold.sociobot.in>. |
| F-3-3 | Added “The sample opens offline after your first visit.” to the initial fact set and added the landing hero to `offline-demo.where`. Phone layout keeps all facts inside 390×844. | `npm run test:e2e -- --grep @claim:offline-demo`; live first-screen test and `.factory/qa-artifacts/polish-3-live-first-screen.png`; <https://inventory-promise-hold.sociobot.in>. |
| F-3-4 | Added a read-only Sample stockroom panel directly after the first screen. It reuses `sampleData()` and renders all three SKUs, the Northline Plumbing hold, and a completed outcome; its link enters `/?demo=1`. | Live browser test `round 3 live first screen has all facts and the data-backed sample preview`; `.factory/qa-artifacts/polish-3-live-landing.png`; <https://inventory-promise-hold.sociobot.in/?demo=1>. |
| F-3-5 | Removed the unsellable landing paid-tier presentation. The only remaining license copy is correctly scoped to existing-license restoration in Settings and Terms; all public status copy says upgrades are unavailable. | `npm run test:e2e -- --grep @claim:pro-checkout-status`; `polish-3-live.spec.ts`; <https://inventory-promise-hold.sociobot.in/terms>. |
| F-3-6 | Replaced every visitor-facing “system of record” use with “inventory or order system,” preserving the legal, warehouse, and storefront limits. | Node contract test `reviewed visitor copy uses one plain term for each concept`; `npm test`; <https://inventory-promise-hold.sociobot.in>. |
| F-3-7 | Defined a hold in README by its observable purpose: it tells coworkers stock may be needed for an order. | Node contract test `reviewed visitor copy uses one plain term for each concept`; `npm test`; [README](../README.md). |

## Prior findings retained and rechecked

Round 3 retained all round 1 and 2 repairs. The clean-clone claim sweep,
`npm test`, `npm run test:e2e:all`, and the live route suite below are the
regression evidence for every row.

| Finding | Current resolution |
| --- | --- |
| F-1-1 | The one-click sample action is registered and enters the isolated seeded demo. |
| F-1-2 | The phone first view contains the job, action, and plain facts. |
| F-1-3 | Static 404 retains the product header, navigation, metadata, art, and footer. |
| F-1-4 | Both 404 implementations use the plain heading “Page not found”. |
| F-1-5 | Demo status says “Sample data”, never live. |
| F-1-6 | Demo has no nonfunctional supervisor-lock control. |
| F-1-7 | Visitor copy consistently uses “audit record”. |
| F-1-8 | README has the task heading “Timed inventory holds for parallel orders”. |
| F-1-9 | README explains roles in plain language. |
| F-1-10 | README calls the callback address the sign-in return address. |
| F-1-11 | README states `/data` and the one-replica SQLite requirement. |
| F-1-12 | Terms keeps the plain fair-use rule. |
| F-2-1 | Demo invalidates delayed live license work and never changes real license state. |
| F-2-2 | Network-derived “Shared live” copy is absent. |
| F-2-3 | Durable shared state is registered and tested against file-backed SQLite. |
| F-2-4 | Setup/deployment guarantees are claims or explicit operator requirements. |
| F-2-5 | The working area is consistently called “inventory holds”. |
| F-2-6 | The combined boundary section is “Limits and data retention”. |
| F-2-7 | License copy names the actual profiles/reminder features and is no longer a landing tier. |
| F-2-8 | README names the sample-stockroom section directly. |
| F-2-9 | README local-run guidance avoids CIAM jargon. |
| F-2-10 | README opening explains the record without audit jargon. |
| F-2-11 | The demo exit action is correctly named “Leave demo”. |
| F-2-12 | Legal home navigation is a link. |
| F-2-13 | Static 404 footer renders the deployed build identifier. |

## Final evidence

- Fresh clone `/tmp/stock-promise-round3-DEuJe5`: `npm ci`, then every one of
  the 22 `.factory/claims.json` commands independently — pass.
- Local gates: `npm test` (3 Vitest, 9 contract, 20 Rust tests), `npm run
  check`, `cargo fmt --all -- --check`, `npm run build`, and `npm run
  test:e2e:all` (21 standard browser tests plus 1 hosted-auth test) — pass.
- Release: `npm run deploy` verified durable `/data`, min/max replicas `1/1`,
  and `/health` build `ae65df265dc1fd7b14b44d7b081033cadb2b40ff`.
- Cold live suite: `EXPECTED_BUILD_SHA=ae65df265dc1fd7b14b44d7b081033cadb2b40ff
  npx playwright test --config .factory/polish-3-live.config.ts` — 2/2 pass,
  including Axe serious/critical scans on home, demo, Privacy, access, Terms,
  and 404. Screenshots: `.factory/qa-artifacts/polish-3-live-first-screen.png`,
  `.factory/qa-artifacts/polish-3-live-landing.png`,
  `.factory/qa-artifacts/polish-3-live-demo.png`, and
  `.factory/qa-artifacts/polish-3-live-404.png`.
- Worker verifier: `.factory/qa-artifacts/verify-url-3/verify.json` records
  HTTP 200, 609 ms load, correct title/lang/one H1/main/alts, no unlabeled
  buttons, and no console errors.
- Lighthouse mobile: performance 99, accessibility 100, best practices 100,
  SEO 100; LCP 1.7 s, CLS 0, TBT 50 ms. Full report:
  `.factory/qa-artifacts/polish-3-lighthouse.json`.

There are no open review findings.
