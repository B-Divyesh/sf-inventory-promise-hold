# Stock Promise — independent verification 4

Work order: `inventory-promise-hold-verify-4`

Candidate: `8e20c412163e67b94148d85552e16a655e60cc84`

Live URL: <https://inventory-promise-hold.sociobot.in>

Verified: 2026-08-30 UTC

## Verdict: FAIL

The candidate and live deployment are not releasable. The core demo and backend
are substantially repaired: all six registered claims pass, the live build is
the requested commit, rate limits work, the one-state deployment is consistent,
and the hold lifecycle is atomic and persistent in local release testing.

Release is still blocked by a dead paid purchase path and an incomplete claims
contract. The advertised $39 Pro purchase returns HTTP 404. Several material
user-facing promises are absent from `.factory/claims.json`, which the supplied
claims contract explicitly makes a failing review. Accessibility and route
metadata also have acceptance-relevant defects.

No product source was changed. This report, the verifier handoff, and four QA
screenshots are the only repository changes.

## Mandatory first gates

### Claims

`.factory/claims.json` exists and contains six entries. The Rust claim commands
passed on their first run. The three browser commands initially could not start
before clean-clone dependency installation (`vite: not found`); after `npm ci`,
all exact commands passed:

- `@claim:demo-isolated` — passed, 1/1.
- `@claim:csv-export` — passed, 1/1.
- `@claim:offline-demo` — passed, 1/1.
- `claim_role_boundary_staff_can_hold_but_not_change_or_resolve_stock` — passed.
- `claim_rate_limit_returns_retry_after_for_excessive_status_requests` — passed.
- `retention_redacts_resolved_hold_personal_fields_but_keeps_audit_rows` — passed.

The checked-in manifest is nevertheless incomplete; see QA4-02.

### Cold first-read

The first screen passes. A fresh 1440×900 browser context showed:

> Hold scarce stock before it is promised twice.
>
> For distributors and resellers taking orders in parallel, Stock Promise
> shows a timed team hold before stock is promised.
>
> Try it with sample data — See a working stockroom immediately.

This plainly answers what it does, who it serves, and what to click first. The
demo is one click away. The cold page returned 200, contacted only its own
origin, and produced no console or page errors. Evidence:
`.factory/verification-evidence/live-first-read-desktop.png`.

## Release-blocking findings

### QA4-01 — High — the advertised Pro purchase is unavailable

The landing page and in-product settings advertise “$39 one-time” Pro operator
profiles and reminders. Both purchase actions link to:

`https://api.sociobot.in/api/v1/products/inventory-promise-hold/checkout`

A fresh GET returned HTTP 404 with:

```json
{"error":"enabled factory product","status":404}
```

The verification endpoint itself is reachable and correctly returned
`{"valid":false,"reason":"invalid"}` for a made-up token, but a customer
cannot buy a license. This is a live integration/deployment failure, regardless
of the local implementation.

### QA4-02 — High — material promises are missing from the claims manifest

Every registered claim passes, but `.factory/claims.json` does not enumerate
all statements a visitor can rely on. Examples include:

- “Timed holds expire automatically.” on the first screen.
- avoiding a duplicate promise / atomic contested-hold behavior in the headline
  and README.
- the append-only audit record in the README and live settings.
- whole-location erasure in the privacy page and README.
- the advertised $39 Pro profiles and on-device reminders.

There are ordinary tests for some underlying behaviors, but these claims do not
have manifest entries with exactly one `@claim:<id>` sandbox test. The supplied
claims acceptance contract says an unlisted claim fails review until the text is
removed or a claim test is added. The three Rust manifest entries also select
ordinary function names rather than tests tagged with their literal
`@claim:<id>` values.

## Other findings

### QA4-03 — Medium — mobile touch targets and 200% text resizing miss the baseline

At 390 px, these visible targets were smaller than the required 44×44 CSS px:

- header “Demo”: 31–38×44 depending on route;
- header “Privacy”: 39×44 on legal routes;
- “View purchase options”: 188.5×35;
- `privacy@sociobot.in`: 161.8×19.

With the root text size forced from 16 px to 32 px in a CSP-bypassed QA context,
the demo document widened from 390 px to 479 px. The “Stock & settings” tab was
laid out from x=395.6 to x=673.8, requiring horizontal navigation. No content
vanished, but this does not meet the supplied 200% resize/mobile reflow baseline.
Evidence: `.factory/verification-evidence/text-200-demo.png`.

Keyboard focus itself is good: every sampled interactive element received a
3 px solid `rgb(255, 213, 138)` outline, the skip link was first, dialog focus
entered the dialog, and Escape closed it.

### QA4-04 — Medium — canonical metadata is wrong or duplicated off the home route

- `/demo` keeps the home canonical URL.
- `/privacy` and `/terms` each render two canonical tags: the original home
  canonical followed by the route-specific canonical.
- Legal routes likewise render both the home and route-specific meta
  descriptions.

Route titles are correct, but duplicate canonicals can cause search engines to
choose the wrong page identity.

### QA4-05 — Low — the footer does not expose a real build identifier

The required footer identity is present, but it says `build current` rather
than a version or build SHA. `/health` does expose the exact SHA, so this does
not affect runtime identity verification.

## Clean-checkout and build evidence

The clone began at the exact candidate commit. `npm ci` installed 141 packages
and reported 0 vulnerabilities.

- `npm test` — passed: 1 Vitest test, 6 deployment-contract tests, 15 Rust
  tests.
- `npm run check` — passed: 0 Svelte/TypeScript findings; Clippy passed with
  warnings denied.
- `cargo fmt --all -- --check` — passed.
- `git diff --check` and `bash -n deploy/*.sh` — passed.
- `npm run build` — passed and produced `dist/`.
- `npm run test:e2e` — passed 8/8.
- `BUILD_SHA=8e20c412163e67b94148d85552e16a655e60cc84 cargo build --release --locked`
  — passed.

No container engine is installed in this verifier image, so a local Docker
image build was not possible. The Dockerfile was inspected, its contract suite
passed, and the exact locked frontend and backend production builds passed.

Production bundle sizes:

- initial application JS: 92,605 bytes raw / 33,021 bytes gzip;
- deferred MSAL chunk: 279,009 bytes raw / about 70.16 KB gzip;
- CSS: 20,569 bytes raw / 5,492 bytes gzip;
- mobile hero: 15,414 bytes; no downloaded web fonts.

## Live identity, function, and persistence evidence

`/health` returned the exact candidate SHA. SHA-256 hashes of the live initial
JS and CSS exactly matched local `dist/` assets.

The independent 390 px demo flow covered invalid quantity 0, a quantity above
availability, holding all nine available units, the 480-minute boundary,
release recovery, conversion, correctly escaped CSV output, zero on-hand stock,
and reset. All passed. Demo traffic stayed same-origin, made no API writes,
stored no cookies, and produced no console, page, or request errors. Evidence:
`.factory/verification-evidence/live-mobile-390-demo.png`.

A clean local release-binary API run proved:

- invalid quantity, 4/481-minute duration, and blank customer return 400;
- two simultaneous two-unit holds against three units return exactly 201/409;
- stock cannot be lowered beneath an active hold;
- release restores availability;
- a three-of-three, 480-minute hold converts to 0/0/0
  on-hand/held/available;
- release and convert events remain in the audit response;
- CSV escapes commas and quotes;
- after stopping and restarting the binary on the same SQLite file, the
  location, inventory, and both outcomes remained present.

The live service was not restarted because verification was not authorized to
mutate deployment state. Instead, 120 concurrent `/api/status` calls from
distinct forwarded clients all returned 200 and `setup_required:true`, with no
mixed state. The empty live location is consistent and ready for its first CIAM
supervisor; it is not the split-state behavior seen in verification 3.

## Live backend, security, privacy, and PWA evidence

- Read allowance observed: requests 1–80 returned 200; 81–90 returned 429.
- Write allowance observed: requests 1–20 reached authorization and returned
  401; 21–30 returned 429.
- Every sampled 429 included `Retry-After: 59`.
- The limiter keys the first `X-Forwarded-For` hop and source middleware wraps
  all `/api` routes; health is exempt.
- 100 concurrent health calls returned 100×200 and the candidate SHA at about
  199 requests/s in this network sample.
- Anonymous bootstrap and writes are denied. An attacker `Origin` receives no
  `Access-Control-Allow-Origin` header.
- Headers include HSTS, nosniff, same-origin referrer policy, restrictive
  permissions policy, and header-only CSP with `frame-ancestors 'none'`.
- HTML/legal routes are `no-cache`; hashed assets are one-year immutable;
  API/health are `no-store`; `sw.js` is no-cache/no-store.
- Clicking live sign-in redirected only to
  `sociobotcustomers.ciamlogin.com/<tenant>/oauth2/v2.0/authorize`, with the
  expected product callback. No human credential was available, so a hosted
  authenticated mutation was not attempted.
- The live service worker controlled `/demo`, updated from `/sw.js`, and
  reloaded the complete sample desk offline without page errors.

## Accessibility and performance evidence

At desktop 1440×1000 and mobile 390×844, `/`, `/demo`, `/privacy`, and `/terms`
each had `lang=en`, one h1, one main landmark, correct route title, no normal-
size horizontal overflow, reduced-motion media active, zero serious/critical
Axe findings, and zero console/page errors.

Lighthouse 13 mobile against the live home page:

- Performance 93; Accessibility 100; Best Practices 100; SEO 100.
- FCP 1.5 s; LCP 1.5 s; CLS 0; TBT 320 ms.
- 170 KiB transferred over 8 requests.

## Final disposition

**FAIL.** Register/enable the paid product or remove the paid offer, complete
the claims manifest and exact claim tests, then fix the undersized targets and
route metadata before repeating independent verification. No deployment or
infrastructure resource was read, changed, or restarted during this work.
