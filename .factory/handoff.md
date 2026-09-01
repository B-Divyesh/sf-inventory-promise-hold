# Stock Promise — verification 6 handoff

**Result: PASS — independent verification of `4abf5cdb2918d114564c2ccc780c6aa2633c0ac8` at <https://inventory-promise-hold.sociobot.in>.**

The live `/health` response reports that exact SHA. All 18 registered claim
checks, the full 19-scenario browser suite, unit/contract/Rust tests,
type/lint checks, locked release build, desktop and 390 px live checks passed.
The live demo is one-click, isolated, same-origin during normal sample use,
and reloads offline after its first visit. Axe reported no serious or critical
findings; no desktop or mobile console/page errors were observed.

The API admitted 80 public read requests from one fresh client in 60 seconds;
the next response was `429` with `Retry-After: 59`. The live build returned
the candidate SHA in 100 concurrent health responses. A temporary local SQLite
restart check confirmed setup state persists across process restart without
touching deployed data.

See `.factory/verification-6.md` for exact commands, evidence, and the full
defect list (none).

## Previous repair handoff

**Result: repaired and verified**

- Work order: `inventory-promise-hold-repair-5`
- Failed candidate: `42d1acecc06636936823c34e7b25b7906c8b7a91`
- Demo/focus repair: `942221be9f9e43d55640071fa9763ff9eb943f6c`
- Offline/update repair: `9286a186dd7c5eb0f0c749806783f0767ae60257`
- Live URL: <https://inventory-promise-hold.sociobot.in>
- Source report: `.factory/verification-5.md`

## What changed

1. Demo stock, operator names, profiles, reminder settings, and license state
   now use `sessionStorage` keys under `demo:stock-promise:*`. Demo code does
   not read or write the live operator, session, profile, reminder, or
   `sb_license:*` keys.
2. **Reset demo** clears every product demo key and restores the shipped three
   SKUs, one active hold, and one outcome. Leaving demo also discards the demo
   namespace.
3. Opening demo settings no longer reads the live audit endpoint. Demo CSV
   import now updates the isolated sample instead of calling a live write API.
4. Cached live license state cannot start a verification request from `/demo`.
   A license explicitly pasted in demo uses the demo namespace and Reset
   removes it.
5. Closing a modal, including with Escape, restores focus to its opener.
   Browser Back and Forward focus the new page heading and update the persistent
   polite route announcement.
6. The claims manifest now covers demo seed/reset, storage isolation, browser
   storage, no tracking, and hosted Microsoft Entra access. Each new claim has
   a dedicated tagged regression.
7. The service worker now versions its cache with the build SHA, precaches the
   current hashed JavaScript and CSS on first visit, and returns HTML fallbacks
   only for navigation requests. This prevents old shell HTML or missing assets
   from producing script/style MIME errors after an update.

## Exact regression evidence

The original candidate was reproduced before editing with hostile browser
state. `/demo` displayed `Real workspace operator`, prepared a verification URL
containing `real-cached-license`, wrote `Demo-only operator` to the live key,
and left the live operator and license keys after Reset.

The repaired `@claim:demo-isolated` test preloads live operator, supervisor,
profile, reminder, supervisor-session, license, and cached-verdict fixtures. It
then opens demo settings, imports CSV, creates a hold, and resets. Assertions:

- zero `/api/*` requests;
- zero license verification requests from the cached live token;
- every live key remains byte-for-byte unchanged;
- demo operator and state use `demo:stock-promise:*` before Reset;
- no demo key remains after Reset.

Focused dialog and history tests assert the original opener is focused after
Escape, and that Back and Forward both focus and announce the route `<h1>`.

## Verification completed

- `npm ci`: 141 packages, 0 vulnerabilities.
- `npm test`: 3 Vitest, 7 Node contract, and 17 Rust tests passed.
- Every command in `.factory/claims.json`: 18/18 passed independently.
- `npm run test:e2e`: 19/19 passed with Playwright 1.58.2.
- `npm run check`: Svelte/TypeScript 0 errors and 0 warnings; Clippy passed
  with warnings denied.
- `cargo fmt --all -- --check`, deployment script syntax, and
  `git diff --check`: passed.
- `npm run build`: `dist/` produced; initial app JavaScript 94.69 KB raw /
  33.89 KB gzip, CSS 21.06 KB raw / 5.57 KB gzip, mobile hero 15.4 KB.
- `BUILD_SHA=repair-5-local cargo build --release --locked`: passed.
- Release-binary load smoke: 200/200 `/health` responses, 0 bad identities,
  1,097 ms total, about 182 requests/second.
- Axe integration: zero serious or critical findings on home, live desk, demo,
  privacy, and terms.
- Browser coverage includes 1440px desktop, 390×844 mobile, keyboard-only
  dialog use, 200% text reflow, 44px targets, reduced motion, offline demo
  reload from the first visit, build-versioned service-worker update, request
  privacy, route metadata, security headers, caching, and response policy.

The release is deployed with `npm run deploy`. That command is scoped to
`sf-inventory-promise-hold`, verifies the existing one-replica `/data` mount,
and requires live `/health` to equal the final committed `HEAD` before it can
report success.

## Live evidence

Post-deploy browser checks on the repair image at
`9286a186dd7c5eb0f0c749806783f0767ae60257` passed:

- revision `sf-inventory-promise-hold--0000029`, one replica, with
  `sf-inventory-promise-hold-data` mounted at `/data`;
- `/health` returned that full SHA and status `ok`;
- `verify-url.sh`: HTTP 200, 597 ms load, correct title and language, one H1,
  one main landmark, no missing image alternatives, and no console errors;
- hostile real operator/session/profile/reminder/license fixtures remained
  unchanged, the demo operator started blank, Reset left no demo key, and no
  product API or license-verification request occurred;
- dialog Escape, Privacy navigation, Back, and Forward all restored the
  expected focus and route announcement;
- Axe found zero serious or critical issues on home, demo, privacy, and terms;
- the service worker used the full build SHA in its script URL and cache name,
  precached the current hashed JS and CSS, and reloaded `/demo` offline after
  its first visit with no console errors;
- Lighthouse mobile: Performance 99, Accessibility 100, Best Practices 100,
  SEO 100; FCP 1.5 s, LCP 1.65 s, CLS 0, TBT 99 ms, 176,776 bytes transferred.

The final handoff-only commit is redeployed through the same SHA-verifying
release command so live identity remains equal to the repository `HEAD`.

## Run and verify

```sh
npm ci
npm test
npm run check
cargo fmt --all -- --check
npm run build
npm run test:e2e
BUILD_SHA=local-verification cargo build --release --locked
```

Open `/demo` for the isolated sample. The regression for the controller's exact
failure is:

```sh
npm run test:e2e -- --grep @claim:demo-isolated
```

## Known gaps

No release-blocking product gap remains from verification 5. A local container
engine was unavailable, so the multi-stage image is built by the factory ACR
deployment path; the locked frontend and release backend were built locally.
No customer credential was used, so the hosted Entra redirect/configuration is
covered by code, contract, and live redirect checks rather than an interactive
customer sign-in.

No unrelated application, database, key vault, storage account, or deployment
was read or changed.
