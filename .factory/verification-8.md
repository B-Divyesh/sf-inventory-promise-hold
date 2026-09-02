# Independent verification 8 — PASS

Verified candidate: `f87062ac9983001b415577f4e70a88299f67b661`  
Live URL: <https://inventory-promise-hold.sociobot.in>  
Verification date: 2026-09-02

## Result

**PASS.** The fresh deployment reports the exact candidate build from
`/health` and met the researched brief's smallest useful job: a reseller can
see sample availability, place a timed hold, receive validation for invalid or
over-available quantities, reset the isolated sample, and export outcomes.
The previous deployment-only concern was not reproduced.

## First read and live product evidence

A cold desktop visit gave this plain first-screen message: “Hold scarce stock
before it is promised twice.” It names distributors and resellers taking
orders in parallel, says that it shows a timed team hold, and presents the
one-click **Try it with sample data** action with “Open a sample stockroom.”
It therefore passes the first-read and demo-sandbox gate.

- `GET /health` returned `200`, `Cache-Control: no-store`, and
  `build_sha: f87062ac9983001b415577f4e70a88299f67b661`.
- `/, /demo, /?demo=1, /privacy, /terms, /robots.txt, /sitemap.xml,
  /manifest.webmanifest, /sw.js` each returned `200`; a made-up route returned
  `404`.
- A fresh Playwright run of `.factory/polish-1-live.spec.ts` passed **5/5**:
  cold 390px first read, one-click demo/reset/isolation, desktop sample labels,
  privacy/terms/history/404 metadata and focus, no serious or critical axe
  findings, service-worker update and offline demo reload, response policy,
  and live build identity.
- Request logging during home, privacy, and demo flows observed only the
  product origin; the registered no-tracking claim test also passed. No browser
  console errors, page errors, or failed requests were observed in the live
  suite.
- Keyboard checks passed: skip link has a visible outline, Enter activates
  controls, invalid supervisor PIN recovers with an announced error, Escape
  closes the hold dialog and returns focus, and history moves focus to the
  destination heading. Reduced-motion and 390px layout checks passed.
- Demo workflow: quantity `0` was rejected by native validation (“Value must
  be greater than or equal to 1.”); a valid one-unit hold then succeeded;
  quantity `999` was rejected against the available maximum; Reset restored
  the single shipped active hold. The flow produced no console errors.
- The public read allowance was **80 requests per client IP per 60 seconds**:
  request 81 returned `429` with `Retry-After: 59`. The source and passing
  claim tests also cover the stricter 20-write/minute path, forwarded-client
  identity, atomic competing holds, expiry, audit immutability, and retention.

## Required claims

All **18/18** exact commands listed in `.factory/claims.json` were run first
from this clean candidate checkout after `npm ci`, and passed. The ten browser
claims were also rerun in the full browser suite; its final result was
`20 passed (45.0s)`. The claims cover demo isolation/reset, no tracking,
documented browser storage, CIAM hosted access, CSV export, offline demo,
role boundary, rate limiting, retention redaction, automatic expiry, contested
stock protection, append-only audit, location erasure, Pro profile/reminder
and restore behavior, unavailable checkout status, and ungated core features.

## Local quality gates

- `npm ci` — passed; no vulnerabilities reported.
- `npm test` — passed: 3 Vitest, 9 Node contract, and 17 Rust tests.
- `npm run check` — passed (`svelte-check` and `cargo clippy -- -D warnings`).
- `cargo fmt --all -- --check` — passed.
- `npm run build` — passed and produced `dist/`.
- `npm run test:e2e` — passed: **20/20** browser scenarios.
- `BUILD_SHA=f87062ac9983001b415577f4e70a88299f67b661 cargo build --release --locked`
  — passed (optimized release profile).

The production frontend build measured 95.02 KB application JS / 33.98 KB
gzip, 279.01 KB lazy chunk / 70.16 KB gzip, and 21.29 KB CSS / 5.61 KB gzip;
the initial application JavaScript stays below the 150 KB gzip site budget.
Live headers include CSP, `X-Content-Type-Options`, same-origin referrer
policy, HSTS, and a restrictive permissions policy. Hashed assets are
immutable for one year; HTML routes revalidate; the service worker is
no-cache/no-store.

## Defects

None found. No release-blocking, high, medium, or low severity defect was
reproduced.

## Scope

No application code, infrastructure, DNS, storage, secrets, or other product
resources were modified during verification. This report and the handoff are
the only candidate changes.
