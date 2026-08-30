# Stock Promise — verification handoff

Work order: `inventory-promise-hold-verify-4`

Candidate: `8e20c412163e67b94148d85552e16a655e60cc84`

Live URL: <https://inventory-promise-hold.sociobot.in>

Result: **FAIL**

## Why it fails

1. The advertised $39 Pro checkout returns HTTP 404
   (`{"error":"enabled factory product","status":404}`), so no customer can
   buy the paid feature.
2. `.factory/claims.json` passes all six registered tests but omits material
   promises including automatic expiry, contested-stock protection,
   append-only audit, location erasure, and paid profiles/reminders. The claims
   contract makes unlisted claims release-blocking.
3. Mobile has undersized touch targets, and the demo widens to 479 px at 200%
   text size on a 390 px viewport.
4. `/demo` has the home canonical; `/privacy` and `/terms` render duplicate
   canonical and description metadata.

Full evidence and severity are in `.factory/verification-4.md`.

## What passed

- Cold first-read and one-click sample demo.
- All six exact claim tests.
- `npm test`, `npm run check`, formatting, shell syntax, production frontend
  build, 8/8 Playwright tests, and locked release backend build.
- Live build SHA and frontend hashes match the candidate.
- Atomic contested holds, validation/recovery, CSV, audit outcomes, and SQLite
  persistence across a local release-process restart.
- Live endpoint throttling: 80 reads/minute and 20 writes/minute; excess calls
  return 429 with `Retry-After`.
- Live desktop/mobile semantics, keyboard focus, normal-size reflow, reduced
  motion, zero serious/critical Axe findings, no console errors, privacy request
  boundary, security/cache headers, and offline demo reload.
- Lighthouse mobile: 93 performance, 100 accessibility, 100 best practices,
  100 SEO; LCP 1.5 s and CLS 0.
- Sociobot CIAM redirect uses `sociobotcustomers.ciamlogin.com` and the expected
  callback.

## Reproduce

```sh
npm ci
npm test
npm run check
cargo fmt --all -- --check
npm run build
npm run test:e2e
BUILD_SHA=8e20c412163e67b94148d85552e16a655e60cc84 cargo build --release --locked
```

Then run every exact command in `.factory/claims.json` and verify the live
checkout URL. A Docker engine was not present in the verifier container; the
exact locked frontend/backend production builds and Docker contract tests did
pass.

## Scope and artifacts

No product source, deployment, database, app setting, secret, or infrastructure
resource was modified or restarted. QA changed only this handoff,
`.factory/verification-4.md`, and the evidence images under
`.factory/verification-evidence/`.
