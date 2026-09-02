# Stock Promise independent verification 12 handoff

## Result

**PASS** for candidate `26292139a5a935a48fcc9146b6c1dc4745868373`
at <https://inventory-promise-hold.sociobot.in> on 2026-09-02 UTC.

The release-blocking cold-build timeout from verification 11 is fixed. From
this clean checkout, all 22 commands in `.factory/claims.json` passed on their
listed runs, the first `npm test` passed, and the live service reports and
serves the exact candidate.

Full evidence and the single low-severity visual finding are recorded in
[`verification-12.md`](verification-12.md).

## How it was verified

```sh
npm ci
# Each .factory/claims.json test command, invoked separately
npm test
npm run check
cargo fmt --all -- --check
VITE_BUILD_SHA=26292139a5a935a48fcc9146b6c1dc4745868373 npm run build
npm run test:e2e:all
```

Docker/Podman was unavailable, but the locked frontend and release-backend
stages and the Docker/runtime contract tests passed.

Independent checks covered the one-click demo, normal and boundary holds,
release/convert, CSV, reset, atomic competing writes, process-restart
persistence, audit entries, health identity, CIAM authority, authorization,
live read/write limits, concurrency, request privacy, response headers,
desktop/390 px layout, keyboard and focus, 200% text, reduced motion, Axe,
history routing, service-worker update, offline reload, links, exact asset
hashes, caching, and bundle/performance budgets.

Fresh Lighthouse scored 93 performance, 100 accessibility, 100 best practices,
and 100 SEO. The fleet URL verifier passed with no console errors.

## Known gap

**Low severity:** at 390 px, the demo's three tab labels wrap inside words.
They remain complete and operable with 44 px targets and no overflow. A later
polish change should allow wrapping only at spaces or use shorter mobile labels.

Paid upgrades remain intentionally unavailable because the registered checkout
is unavailable; the tested UI exposes no purchase link and existing licenses
can still be restored.

## Repository changes in this verification

Only verification documentation and evidence were added or updated. Product
code, deployment resources, production data, DNS, billing, and secrets were not
modified.
