# Independent verification 7 — FAIL

Date: 2026-09-01 (UTC)  
Candidate tested: `b4fe5c74d419d83b35fe39514770a5ca3ecfc586`  
Live URL: <https://inventory-promise-hold.sociobot.in>

## Release decision

**FAIL — release blocking.** The live deployment does not identify as the
candidate. A fresh `GET /health` at 23:33 UTC returned HTTP 200 and:

```json
{"build_sha":"f87062ac9983001b415577f4e70a88299f67b661","status":"ok"}
```

The required candidate SHA is `b4fe5c74d419d83b35fe39514770a5ca3ecfc586`.
The deployment therefore cannot be accepted as this candidate, even though the
observed application behavior was otherwise healthy.

## First read and live product evidence

Cold desktop and 390 px sessions loaded HTTP 200 with no console or page
errors. The first screen says, in plain words, **“Hold scarce stock before it
is promised twice.”** It names distributors and resellers taking parallel
orders, and exposes one visible **“Try it with sample data”** action with the
adjacent explanation “Open a sample stockroom.” The action is one click and
leads to the isolated demo. This passes the first-read and demo requirement.

On live `/demo`, the sample contained three SKUs and one active hold. Quantity
`0` was rejected by native validation (“Value must be greater than or equal to
1.”); changing it to `1` created a hold; **Reset demo** restored three SKUs and
one hold. Cold home, demo, privacy, and terms sessions made same-origin
requests only, set no cookies, had no horizontal overflow at 390 px, and had
no serious or critical axe findings. Each route had exactly one `h1`.

The live response policy was observed as follows: home `no-cache, must-
revalidate`; hashed JS `public, max-age=31536000, immutable`; service worker
`no-cache, no-store, must-revalidate`; health and API `no-store`. Responses
included CSP with `frame-ancestors 'none'`, HSTS, `nosniff`, same-origin
Referrer-Policy, and the restrictive Permissions-Policy. `/api/auth/config`
returned `{"mode":"ciam"}`; the candidate source configures only the
Sociobot customer authority (`sociobotcustomers.ciamlogin.com`).

The public API allowance was independently exercised using one forwarded client
identity. Requests 1–80 to `/api/status` returned 200; request 81 returned
HTTP 429 with `Retry-After: 57` and a JSON error. This confirms the documented
rate limiting behavior, with an observed public allowance of 80 requests per
window.

## Candidate checks from a clean detached checkout

`npm ci` completed without vulnerabilities. Every declaration in
`.factory/claims.json` was run separately through its declared entry point and
passed: `demo-isolated`, `demo-seed-reset`, `no-tracking`, `browser-storage`,
`hosted-access`, `csv-export`, `offline-demo`, `role-boundary`, `rate-limit`,
`retention-redaction`, `automatic-expiry`, `contested-stock-protection`,
`append-only-audit`, `location-erasure`, `pro-profiles-reminders`,
`pro-license-restore`, `pro-checkout-status`, and `core-features-no-pro`
(18/18).

The following also passed against the candidate:

```sh
npm test                         # 3 Vitest + 8 Node contracts + 17 Rust tests
npm run check                    # svelte-check 0 errors/warnings; clippy -D warnings
cargo fmt --all -- --check
npm run build                    # dist/ produced
BUILD_SHA=b4fe5c74d419d83b35fe39514770a5ca3ecfc586 cargo build --release --locked
npm run test:e2e                 # 20/20 Playwright scenarios
bash -n deploy/*.sh
```

The production frontend build measured 95.02 kB raw / 33.98 kB gzip for its
initial application JS, plus 21.29 kB raw / 5.61 kB gzip CSS. This is within
the static first-load JS and CSS budgets. The full e2e run covered the normal
hold lifecycle, CSV, invalid-PIN recovery, keyboard/dialog/history focus,
mobile, text resize, reduced motion, offline reload, headers, metadata, and
accessibility. Docker was not installed in the verifier container, so an
image build could not be run; the locked release binary build above passed.

## Defects

| Severity | Finding | Evidence / required resolution |
| --- | --- | --- |
| Release blocker | Candidate is not the deployed build. | Live `/health` reports `f87062ac9983001b415577f4e70a88299f67b661`, not candidate `b4fe5c74d419d83b35fe39514770a5ca3ecfc586`. Deploy the exact candidate and rerun identity verification. |

No product-code changes were made during this verification. No cloud resource,
other service, secret, storage account, staging slot, DNS, or billing setting
was read or modified.
