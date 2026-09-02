# Independent verification 9 — PASS

Verified candidate: `fea95f207aaae2bfd98db15ea3c75c03760b8dab`  
Live URL: <https://inventory-promise-hold.sociobot.in>  
Verification date: 2026-09-02

## Verdict

**PASS.** Fresh local and live evidence confirms that this candidate delivers
the brief's temporary, shared stock-promise workflow. The live `/health` build
identity is exactly the candidate SHA, and the deployed frontend entry HTML,
CSS, application chunk, and shared chunk byte-match a `BUILD_SHA=<candidate>`
production build from this checkout.

## First-read and demo gate

A cold desktop visit plainly says: “Hold scarce stock before it is promised
twice.” It identifies “distributors and resellers taking orders in parallel,”
explains that it shows a timed team hold before stock is promised, and presents
the first action **Try it with sample data** with the adjacent explanation
“Open a sample stockroom.” The first screen therefore answers what it does,
who it is for, and what to click first. The one-click action opened the
isolated sample stockroom at `/?demo=1`, with the persistent “Demo — sample
data, nothing is saved” banner, Reset demo, and Leave demo controls.

## Required claims

After `npm ci`, all **20/20** entries in `.factory/claims.json` passed from
the clean checkout:

- 10 exact Rust claim commands: hosted CIAM access; one-time supervisor setup;
  durable shared storage; role boundary; rate limiting; retention redaction;
  expiry; atomic contested-stock protection; append-only audit; location
  erasure.
- 10 browser demo claim tests: isolation, seed/reset, no tracking, documented
  browser storage, CSV export, offline demo, Pro reminders/profiles and
  license restore, unavailable checkout, and free core features.

The full local browser suite also passed **20/20** (`test-results/.last-run.json`
reported `{"status":"passed","failedTests":[]}`).

## Functional, privacy, accessibility, and deployment evidence

- Local quality gates passed: `npm test` (3 Vitest, 9 contract, 19 Rust),
  `npm run check` (Svelte check and warning-denied Clippy),
  `cargo fmt --all -- --check`, and `npm run build`.
- `BUILD_SHA=fea95f207aaae2bfd98db15ea3c75c03760b8dab npm run build` produced
  `dist/`; the live HTML and its three generated assets had matching SHA-256
  digests. The live `/health` body was
  `{"build_sha":"fea95f207aaae2bfd98db15ea3c75c03760b8dab","status":"ok"}`.
- Cold home, desktop demo, and 390 px reduced-motion demo produced no console
  errors, page errors, or external requests. Fresh request logs contained only
  `https://inventory-promise-hold.sociobot.in`; no cookies were set in the
  no-tracking claim flow.
- Playwright Axe checks on live home and demo at desktop and 390 px found zero
  serious or critical violations. At 390 px the demo width was exactly 390 px
  (no horizontal overflow); reduced motion was active. The local suite also
  covers 44 px targets, 200% text reflow, keyboard skip link, dialog Escape
  focus restoration, history focus announcements, service-worker update, and
  offline demo reload.
- In the live demo, quantity `0` was blocked by native validation
  (“Value must be greater than or equal to 1.”) without changing the hold
  count. A valid one-unit hold succeeded and Reset demo restored the seeded
  one-hold sample. No error reached the console.
- `/api/auth/config` returned `{"mode":"ciam"}`. The passing hosted-access
  claim verifies the Sociobot Microsoft Entra External ID default tenant.
- Live headers include CSP with response-header `frame-ancestors`, HSTS,
  `X-Content-Type-Options`, same-origin referrer policy, and restrictive
  Permissions-Policy. HTML routes revalidate, hashed JS is immutable for one
  year, `/sw.js` is no-cache/no-store, and unknown routes return 404.
- Rate-limit test from a dedicated forwarded client IP observed 80 successful
  public status requests in the 60-second read window; request 81 returned
  `429` with `Retry-After: 57`. The passing claim also covers the stricter
  write path.
- Production frontend output is 95.56 KB raw / 34.10 KB gzip for the entry
  JS, 279.01 KB raw / 70.16 KB gzip for its shared chunk, and 21.29 KB raw /
  5.61 KB gzip CSS. Total startup JS gzip is about 104 KB, within the 150 KB
  site budget.

## Limitation and defects

No product defect was found: **0 blocker, 0 high, 0 medium, 0 low**.

The verification container does not provide the `docker` executable, so the
Docker image build itself could not be invoked here. This is an environment
limitation, not a deployment-only failure: the exact locked Rust release build
was run locally and the already deployed live frontend and backend identity
were independently matched to the candidate. No product code, production
data, infrastructure, DNS, secrets, or unrelated resources were changed.
