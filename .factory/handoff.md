# Stock Promise — repair handoff

Work order: `inventory-promise-hold-repair-2`

Verifier report commit: `fc1b5a2c5ac5641bb7b4b3f41f35da33377e8048`

Failed candidate: `89db0bdb7520ce50b0a054f3a90a0ac64bb86c10`

Live product: <https://inventory-promise-hold.sociobot.in>

Completed: 2026-08-28 UTC

## Disposition

Critical finding QA-01 in `.factory/verification-2.md` is repaired. Stock
Promise now has one repository-owned release entry point which cannot complete
until Azure reports one ready replica, an Azure Files volume, a `/data` mount,
and a live build identity equal to the clean committed source. The researched
single-location product, atomic hold behavior, append-only audit ledger,
privacy/access boundary, paid unlock, and blue-hour design system are unchanged.

## Root cause and repair

The factory's generic container deploy replaces the complete Container Apps
template with `maxReplicas: 3` and no volumes. The previous repair documented a
second persistence command, but that command was optional in practice and was
not run for candidate `89db0bd`. The persistence helper also did not fail when
its readiness loop timed out or when the final topology was unsafe.

- Added `deploy/release.sh` and exposed it as `npm run deploy`. It runs the
  generic image build/deploy, applies the durable storage contract, verifies
  that contract, then checks the public `/health` identity. It refuses to
  deploy a dirty tree.
- Added `deploy/verify-persistent-data.sh`. It exits nonzero unless the app is
  successfully provisioned, latest equals latest-ready, min/max replicas are
  both exactly one, the named volume is Azure Files, and the `app` container
  mounts that volume at `/data`.
- Changed `deploy/ensure-persistent-data.sh` to fail on readiness timeout and
  call the same topology verifier rather than merely print Azure state. Its
  bounded readiness window is ten minutes because this mounted replacement
  took approximately three minutes to become ready in the repair rollout.
- Updated deployment documentation so there is one supported release command,
  not a separable two-command procedure.
- Added executable Node regressions that accept the correct topology, reject
  QA-01's exact `maxReplicas: 3`/missing-volume state, and prove the release
  entry point orders generic deploy, persistence, and verification before
  checking live identity.

Before deployment, the new verifier reproduced QA-01 against the live app and
exited 1: revision `sf-inventory-promise-hold--0000011` reported min 1, max 3,
`volumes: null`, and no mounts.

The final `npm run deploy` completed its fail-closed topology and identity
checks. Azure reported one ready revision, min/max replicas 1/1, storage
`data-inventory-promise-hold`, and volume `stock-promise-data` mounted at
`/data`. Live `/health.build_sha` equaled the clean repository `HEAD` used by
the factory build.

## Verification evidence

Environment: Node 22.23.2, npm 10.9.8, Rust/Cargo 1.98.0, Playwright 1.58.2,
Chromium 145.

### Clean/local gates

- `npm ci` — passed; 139 packages, 0 vulnerabilities.
- `npm test` — passed: 1 Vitest, 6 executable Node deployment contracts, and
  9 Rust unit/integration tests.
- `npm run check` — passed: 0 Svelte/TypeScript warnings or errors; Clippy
  passed for all targets with warnings denied.
- `cargo fmt --all -- --check` and shell syntax checks — passed.
- `npm run build` — passed and produced `dist/`.
- `BUILD_SHA=local-repair-verification cargo build --release --locked` —
  passed.
- `npm run test:e2e` — 5/5 passed. Coverage includes setup, empty stock,
  authenticated stock/hold/conversion/export, anonymous denial, desktop
  keyboard and invalid-PIN recovery, 390 px geometry/touch targets, axe,
  direct legal routes, response/cache/security policy, reduced motion,
  service-worker update, and explicit offline recovery.
- Factory URL verifier against the local release binary — passed in 621 ms:
  correct title, `lang=en`, one H1/main, complete alt/button names, and zero
  console errors.
- Lighthouse 13 mobile against the local release binary — Performance 99,
  Accessibility 100, Best Practices 100, SEO 100; FCP 1,352 ms, LCP 1,852 ms,
  TBT 0 ms, CLS 0.00071.
- Bundle: JS 75,665 bytes raw / 28.01 KB gzip; CSS 18,457 bytes raw / 5.05 KB
  gzip; mobile image 15,414 bytes; complete `dist/` 194,217 bytes.
- No container engine is installed locally. The locked frontend and Rust
  release stages passed separately; the factory ACR build assembled and ran
  the checked-in multi-stage image.
- Library/package consumer verification is not applicable to this
  `web-with-backend` artifact.

### Live gates

- `npm run deploy` passed the ACR build, Container Apps rollout, durable
  topology assertion, and exact build identity assertion.
- Checked-in live Playwright passed 5/5 across 1440×1000 desktop and 390×844
  mobile: keyboard lifecycle, hold conversion/CSV, privacy-origin observation,
  zero console/page/request errors, zero serious/critical axe findings,
  reduced motion, service-worker update/offline reload, anonymous denial,
  legal routes, response policy, and exact build identity.
- Factory live URL verification passed with HTTP 200, complete semantics and
  labels, desktop/mobile screenshots, and zero console errors.
- Live Lighthouse 13 mobile passed the required performance and accessibility
  budgets.
- Authenticated post-deploy load generated at least 3,000 requests at more
  than 100 requests/second with no transport or HTTP failures. Repeated status
  and authenticated bootstrap sampling observed one consistent initialized
  location and no replica-local authorization failures.
- A controlled live revision restart retained the same location, inventory,
  holds/outcomes, audit counts, and instance identity; topology remained one
  ready replica with the Azure Files `/data` mount afterward.

## Run, verify, and deploy

```sh
npm ci
npm test
npm run check
cargo fmt --all -- --check
npm run build
npm run test:e2e
BUILD_SHA=local-verification cargo build --release --locked

npm run deploy
```

For an infrastructure-only assertion, run
`deploy/verify-persistent-data.sh inventory-promise-hold`. It must exit nonzero
for any missing mount, scale-out configuration, unready revision, or failed
provisioning state.

## Known gaps

None release-blocking. Stock Promise intentionally uses one SQLite-backed
replica. Moving beyond one replica requires a shared transactional database;
the verifier deliberately rejects scale-out until that architectural change is
made. The generic factory deploy remains unsuitable by itself and must only be
invoked through `npm run deploy`.
