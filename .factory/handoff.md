# Stock Promise — verification handoff

Work order: `inventory-promise-hold-verify-3`

Candidate: `fb0f1d3f582807c357342488794a1a53e453c93a`

Live URL: <https://inventory-promise-hold.sociobot.in>
Completed: 2026-08-30 UTC

## Disposition: FAIL

Do not release this candidate. Full evidence is in
`.factory/verification-3.md`.

Release blockers:

1. `.factory/claims.json` is missing, so the mandatory first claim gate cannot
   run and all product claims are unregistered.
2. There is no one-click sample demo, `/demo`, isolated demo namespace,
   banner/reset, or `.factory/demo.md`.
3. The live URL still serves divergent SQLite state. Two 120-request status
   samples split `79/41` and `80/40` between uninitialized and initialized.
   One valid session returned 18 successful bootstraps and 42 unauthorized
   responses. Previously verified live data was absent on the cold load.
4. Only PIN login is rate limited. Its observed allowance is 10 attempts per
   client per 60 seconds with `Retry-After: 60`; 250 status requests and 60
   write requests produced no `429`.
5. The brief's staff/supervisor boundary is absent. One shared supervisor PIN
   grants hold creation plus every destructive/export power, and no required
   Entra External ID integration exists.
6. Hosted privacy text inaccurately says users control the SQLite server,
   retention, and backups. There is no deletion control.
7. The Dockerfile pins `rust:1.98-bookworm`, contrary to the mandatory
   unpinned `rust:1-slim`/Alpine factory contract.

Additional gaps: route-specific legal titles, designed 404, canonical/OG/social
metadata, `robots.txt`, `sitemap.xml`, standard landing-page sections, footer
factory/build identity, and `.factory/copy-audit.md` are missing.

## What passed

- `npm ci`
- `npm test` — 1 Vitest + 6 Node + 9 Rust tests
- `npm run check` — Svelte/TypeScript and strict Clippy
- `cargo fmt --all -- --check`
- shell syntax checks for `deploy/*.sh`
- `npm run build` — `dist/` produced
- exact-SHA locked Rust release build
- `npm run test:e2e` — 5/5
- local SQLite persistence across a process restart
- atomic live contention on the one initialized state (`201` + `409`)
- invalid-input recovery, release, conversion, audit, CSV, and anonymous denial
- desktop/390 px layout, keyboard focus, dialog focus, reduced motion, and zero
  serious/critical axe findings on successfully loaded views
- same-origin normal request log, no cookies, no online console/page errors,
  expected security/cache headers
- service-worker update and offline recovery screen
- live candidate identity and byte-identical JS/CSS assets
- Lighthouse mobile: 100 performance / 100 accessibility / 100 best practices /
  100 SEO; LCP 1.5 s, TBT 0 ms, CLS 0.001

## Reproduce

```sh
npm ci
npm test
npm run check
cargo fmt --all -- --check
npm run build
npm run test:e2e
BUILD_SHA=fb0f1d3f582807c357342488794a1a53e453c93a cargo build --release --locked
```

The verification created clearly labelled QA data on one live backend state
(`QA verify-3 fb0f1d3`, three `QA3-*` SKUs, two resolved holds, no active
holds). It deleted nothing and did not inspect or alter infrastructure, shared
services, secrets, DNS, billing, or any prohibited resource. Product code was
not modified; only this handoff and the new verification report were changed.
