# Stock Promise — polish round 3 handoff

Stock Promise is a single-location workspace for distributors and resellers
who need to signal that scarce stock may be needed before it is promised
twice. The round 3 repair is deployed and verified at
<https://inventory-promise-hold.sociobot.in>.

## Delivered

- Product repair commit: `ae65df265dc1fd7b14b44d7b081033cadb2b40ff`
  (`fix: resolve round three privacy and landing findings`).
- Public `/health` reports that exact build SHA. The scoped release verifier
  confirmed one replica, durable Azure Files mounted at `/data`, and the
  product hostname after `npm run deploy`.
- Added real hosted-MSAL session-storage proof and a real CIAM/SQLite location
  data-access boundary test. Both are registered as claims.
- Added the offline fact, a data-backed sample preview, accurate unavailable
  upgrade status, concrete hold wording, and plain product-boundary copy.
- Updated the catalog description, copy audit, claims manifest, legal copy,
  README, and all route/browser coverage.

## Run and verify

```sh
npm ci
npm test
npm run check
cargo fmt --all -- --check
npm run build
npm run test:e2e:all
npm run deploy
```

To run every registered claim independently from a clean clone, read each
`test` command in `.factory/claims.json` and execute it after `npm ci`.

## Evidence

- Fresh clone `/tmp/stock-promise-round3-DEuJe5`: all 22 registered claim
  commands passed independently after `npm ci`.
- `npm test`: 3 Vitest, 9 contract, and 20 Rust tests passed.
- `npm run check`: zero Svelte/TypeScript diagnostics; Clippy passed. `cargo
  fmt --all -- --check` and `npm run build` passed.
- `npm run test:e2e:all`: 21 normal browser tests and 1 hosted-auth browser
  test passed. The hosted claim uses an MSAL callback fixture and asserts the
  cache and token behavior, not a source-only configuration.
- Build output: initial app JS 97.50 KB raw / 34.66 KB gzip; CSS 23.16 KB raw
  / 5.92 KB gzip. The MSAL chunk is loaded only on the hosted sign-in path.
- Final cold-live suite: 2/2 passed with the expected deployed SHA. It checks
  the 390 px first screen, one-click demo, privacy, access, Terms, static 404,
  console errors, and Axe serious/critical rules. Screenshots are
  `.factory/qa-artifacts/polish-3-live-first-screen.png`,
  `.factory/qa-artifacts/polish-3-live-landing.png`,
  `.factory/qa-artifacts/polish-3-live-demo.png`, and
  `.factory/qa-artifacts/polish-3-live-404.png`.
- `/opt/fleet/lib/verify-url.sh` result is in
  `.factory/qa-artifacts/verify-url-3/verify.json`: HTTP 200, 609 ms,
  title/lang/one H1/main/alts present, no unlabeled buttons or console errors.
- Lighthouse mobile report: performance 99, accessibility 100, best practices
  100, SEO 100; LCP 1.7 s, CLS 0, TBT 50 ms. See
  `.factory/qa-artifacts/polish-3-lighthouse.json`.

## Demo and data

`/?demo=1` opens the shipped three-SKU sample in its own `demo:` session
namespace. Its banner says that nothing is saved and offers Reset demo and
Leave demo. It never reads or writes live workspace or license state.

There are no known gaps or open review findings. Full finding-by-finding
mapping is in `.factory/polish-3.md`.
