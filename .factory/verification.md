# Stock Promise — independent verification

**Result: FAIL**

- Work order: `inventory-promise-hold-verify-1`
- Candidate: `e826a4523441a78eeaf60f864a77ec0983f367be`
- Live URL: <https://inventory-promise-hold.sociobot.in>
- Verified: 2026-08-28 UTC
- Acceptance contract: `.factory/brief.json`, `.factory/design.md`, repository
  `AGENTS.md`, and the injected accessibility/backend/performance requirements

The candidate builds and its core transaction logic works locally, but the live
product is not releasable. The deployed installation lost all shared state
during this verification, and it does not report a candidate build identity.
The public deployment also exposes operational data and hold creation without a
staff access boundary.

## Release-blocking defects

### Critical — live shared state was lost during verification

At 04:25:39 UTC, the live installation was named `QA candidate e826a45` and
contained four inventory items. A fresh concurrency check created SKU
`QA-C042539`; simultaneous one-unit holds returned exactly `409` and `201`, and
the winning hold was released. A subsequent bootstrap showed the item, its
released outcome, and the original location data.

By 04:27 UTC, without any delete/reset operation from the verifier,
`GET /api/bootstrap` returned:

```json
{"setup_required":true,"location_name":null,"inventory":[],"active_holds":[],"recent_outcomes":[]}
```

The product has no route that deletes settings, inventory, or the audit ledger.
The observed change therefore requires the deployed database to have been
replaced or reset. This fails the shared coordination, append-only audit, and
persistence contract. In contrast, the candidate release binary retained its
location, inventory, converted outcome, and audit records across a controlled
local process restart when the same SQLite file was reused.

### High — public operational reads and hold creation have no staff access boundary

On the internet-facing live origin, `GET /api/bootstrap` requires no
authentication and returns SKUs, availability, customer/order references,
operator names, notes, active holds, and recent outcomes. `POST /api/holds` also
requires no staff session; the live concurrency check created a hold with no
Authorization header. Only supervisor mutations are guarded.

This model can be appropriate on the brief's trusted local network, but it is
not privacy- or integrity-safe on a public hosted URL. Any internet client can
read operational references and consume available stock with new holds. Add a
staff access boundary or restrict the deployment to the intended trusted local
network before release.

### High — deployed backend identity is unverifiable

`GET /health` returned:

```json
{"build_sha":"development","status":"ok"}
```

The Dockerfile does not declare the mandatory `ARG BUILD_SHA=dev` or persist a
factory-supplied build argument into the runtime image. The frontend is a
byte-for-byte candidate match: live `index.html`, JS, and CSS matched the clean
candidate build, including SHA-256
`ed65713ddef3a524a1140744404c22cc5b3f571114f73858bd29f2b5d314e711`
for `index-CxoJ_ok7.js`. The backend cannot be confirmed as the candidate, so
the deployment identity requirement fails.

## Other defects

### Medium — supervisor PIN checks are not rate limited

The public `/api/session` route accepts a 6-digit minimum PIN. It applies a
450 ms delay to an incorrect attempt, but the router has no request-rate layer
and concurrent attempts are not capped. Because a supervisor session permits
stock edits, conversions, audit reads, and CSV export, server-side per-client
and installation-wide rate limits are required.

### Medium — legal deep links return HTTP 404

Direct `HEAD /privacy` and `HEAD /terms` both returned `404` with the SPA HTML
body. Client-side navigation renders both pages, but direct visits, crawlers,
and no-script access receive an error response. These required pages should
return `200`.

### Medium — production caching policy is absent

The HTML, service worker, hashed JS/CSS, and responsive images returned no
`Cache-Control` header. Hashed assets therefore do not receive the required
long-lived immutable policy, while HTML and `sw.js` have no explicit
revalidation policy. Authenticated audit/export responses also have no explicit
`no-store` policy in the service code.

### Low — two mobile legal targets are shorter than 44 px

At a 390 px viewport, the footer Privacy and Terms links measured `44×18` and
`36×18` CSS px. They need a 44×44 px interactive area. Axe did not flag this
geometry issue.

### Low — browser hardening headers are incomplete

Live responses include CSP, `X-Content-Type-Options: nosniff`, and
`Referrer-Policy: same-origin`; an unrelated-origin CORS preflight receives no
allow-origin header. Responses do not include HSTS or Permissions Policy.

## Clean checkout and build evidence

A detached worktree was created directly at the candidate SHA. Before install,
`git status --short --branch` showed only `## HEAD (no branch)`.

Environment: Node `22.23.2`, npm `10.9.8`, Rust/Cargo `1.98.0`, Playwright
`1.58.2`.

- `npm ci` — passed; 139 packages installed, 0 vulnerabilities reported.
- `npm test` — passed: 1/1 Vitest and 3/3 Rust tests.
- `npm run check` — passed: 0 Svelte/TypeScript errors or warnings; Clippy
  passed with `-D warnings`.
- `npm run build` — passed and produced `dist/`.
- `cargo build --release --locked` — passed.
- `npm run test:e2e` — passed, 2/2 Playwright tests at 390×844.
- Clean worktree remained free of tracked or untracked changes after the gates.

No Docker-compatible builder is installed in the verifier image, so the final
container assembly could not run. Both locked build stages and the optimized
runtime binary were exercised independently. The Dockerfile was reviewed
against the mandatory no-`.git`, non-root, default-port, and build-identity
contracts; build identity is defective as recorded above.

Build sizes:

- JS: 74,287 bytes raw / 27.69 KB gzip (budget: 200 KB)
- CSS: 18,294 bytes raw / 5.01 KB gzip (budget: 50 KB)
- Mobile stockroom image: 15,414 bytes (budget: 300 KB)
- Full `dist/`: 192,677 bytes; no web-font payload

## Functional and backend evidence

Independent release-server checks covered setup, inventory, holds, resolution,
expiry, export, concurrency, and persistence:

- Invalid setup PIN returned `400`; unauthenticated inventory and resolution
  returned `401`; invalid SKU returned `400`.
- Quantity `0` and `1,000,001`, plus duration `4` and `481` minutes, each
  returned `400` with recovery text.
- Two simultaneous 2-unit holds against 3 on hand returned `201` and `409`;
  bootstrap showed 2 held and 1 available.
- Lowering on-hand below the active quantity returned `409`.
- Conversion reduced on-hand, repeat resolution returned `409`, and CSV export
  contained the converted outcome.
- A due hold was swept on bootstrap, marked `expired` by `Clock`, returned its
  unit to availability, and appended `hold.expired` to the audit log.
- Local process restart with the same database retained setup, inventory,
  outcomes, and audit data.
- The live concurrency check likewise returned `201` and `409`; immediately
  afterward, bootstrap showed one held unit and zero available.
- Live load smoke on `/api/bootstrap`: 3,189 requests, 637.8 requests/s average,
  92 ms p99, 0 errors, 0 timeouts, and 0 non-2xx responses.

The repository tests also verify that competing holds are atomic, conversion
deducts stock, and database triggers reject audit updates/deletes.

## Browser, accessibility, privacy, and PWA evidence

Fresh live Playwright checks passed 4/4 at 1440×1000 and 390×844:

- normal hold → convert → outcome → CSV flow;
- invalid supervisor PIN error and successful recovery;
- keyboard-only skip link, dialog entry, buttons, and visible focus;
- one H1, `lang=en`, main landmark, labelled controls, and no horizontal
  overflow at 390 px;
- reduced-motion media query active;
- 0 serious/critical axe findings on desktop and mobile;
- 0 console errors, page errors, or failed requests;
- normal first load contacted only the product origin;
- service worker controlled the page, `registration.update()` completed, and
  an offline reload presented the offline/error recovery UI.

The factory URL verifier passed with HTTP 200, load 708 ms, title, language,
one H1, main, complete image alt attributes, labelled buttons, and no browser
errors.

Privacy review found no analytics, advertising, CDN scripts, third-party fonts,
cookies, or normal-load third-party requests. Supervisor tokens use
`sessionStorage`; operator preferences and optional license state use
`localStorage`. The CSP limits normal connections to self and the documented
Sociobot billing origins. The unauthenticated public operational API remains
the material privacy defect.

## Performance evidence

Lighthouse 13 mobile against the live URL:

- Performance: **94**
- Accessibility: **100**
- Best Practices: **100**
- SEO: **100**
- LCP: **1,829 ms**
- CLS: **0.00235**
- TBT: **269.5 ms**
- FCP: **1,401 ms**

The scored budgets pass. Lighthouse did not provide a lab INP value.

## Required disposition

Do not release this candidate as verified. Preserve the deployed SQLite data
across restarts, add an appropriate staff access boundary for the public host,
and return the exact backend build SHA from `/health`. Then correct the legal
route statuses and caching policy and rerun the full verification suite.
