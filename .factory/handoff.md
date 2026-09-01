# Stock Promise — repair handoff

Work order: `inventory-promise-hold-repair-4`
Base verifier report: `6564e8b7d65c2df9d89f4111040e883c7ca82f73` / candidate `8e20c412163e67b94148d85552e16a655e60cc84`
Verified locally: 2026-09-01 UTC

## Repaired release blockers

- Removed every product purchase link to the independently recorded unavailable
  checkout. The product now plainly says that new Pro purchases are temporarily
  unavailable, while existing licenses can be restored and verified in settings.
  A regression test asserts that no `/checkout` link is offered. The shared
  checkout endpoint was not queried or changed because it is operator-owned.
- Expanded `.factory/claims.json` from six to fourteen tested claims. It now
  covers automatic expiry, contested-stock protection, append-only audit,
  location erasure, Pro profiles/reminders, license restore, core features
  without Pro, and the honest checkout status. Rust claim tests carry literal
  `@claim:<id>` tags.
- Added lifecycle regressions for automatic expiry (including its audit row),
  atomic competing holds, immutable/redacted audit data, and full location
  erasure with the append-only trigger restored.
- Fixed mobile target sizing and 200% text reflow. Header navigation, legal
  links, profile chips, and the compact supervisor action meet 44px targets;
  the mobile desk tabs now reflow in a three-column grid without widening a
  390px document at 200% root text.
- Replaced duplicated Svelte head tags with one controlled metadata set.
  `/`, `/demo`, `/privacy`, `/terms`, and `/404` each have one canonical,
  one description, and a route-specific title. The footer now shows the build
  identifier supplied at build time.

## Verification evidence

Clean dependency install completed with `npm ci` (141 packages, 0 reported
vulnerabilities). These commands passed after the repair:

```sh
npm test                         # 3 Vitest, 7 contracts, 17 Rust tests
npm run check                    # Svelte/TypeScript and clippy -D warnings
cargo fmt --all -- --check
bash -n deploy/*.sh
git diff --check
npm run build                    # dist/; initial JS 33.32 KiB gzip, CSS 5.57 KiB gzip
npm run test:e2e                 # 14/14 Playwright checks
```

Every exact command in `.factory/claims.json` was also run. That is seven
isolated browser claim checks and seven isolated Rust claim checks.

Browser coverage includes 390px mobile, a 1440px keyboard desk flow, visible
focus, dialogs and Escape, serious/critical Axe checks, 44px target checks,
200% text reflow, reduced motion, offline demo reload, service-worker update,
privacy request boundary, route metadata, cache/security headers, and local
response policy. The repository has no `verify-url.sh`; its checks are covered
by the Playwright route and Axe integration.

Local Lighthouse desktop on the locally served production frontend
(`/tmp/stock-promise-lighthouse-final.json`) reported Performance 100,
Accessibility 100, Best Practices 100, SEO 100, LCP 0.5 s, and CLS 0. A local
`/health` check returned the supplied build id.

## Run and deploy

```sh
npm ci
npm test
npm run check
npm run build
npm run test:e2e
BUILD_SHA=<commit-sha> cargo build --release --locked
npm run deploy
```

The container serves `PORT` (default 8080), uses SQLite at
`/data/stock-promise.db`, and the scoped release script verifies the one-replica
`/data` mount plus `/health` build identity.

## Known operational gap

The shared Pro checkout remains unavailable and is explicitly out of this
product work order. Stock Promise does not send customers to it or claim that a
new purchase can be completed. Existing license restoration remains available;
core holds and CSV export remain usable without Pro.

No prohibited service, database, key vault, or shared resource was read,
changed, or restarted during this repair.
