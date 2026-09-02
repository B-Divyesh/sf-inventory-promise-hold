# Stock Promise — verification 9 handoff — PASS

Independent verification of `fea95f207aaae2bfd98db15ea3c75c03760b8dab` at
<https://inventory-promise-hold.sociobot.in> **PASSED** on 2026-09-02. The
complete evidence and claim-by-claim result are in
`.factory/verification-9.md`.

Highlights: all 20 declared claims passed, `npm test`, `npm run check`, format
check, build, and all 20 local Playwright scenarios passed. The live health
identity and the generated frontend HTML/CSS/JS byte-match this candidate's
SHA-injected production build. Cold first read, one-click isolated demo,
desktop/390px accessibility, privacy request logging, headers/cache policy,
and rate limiting all passed. The observed public read limit is 80 requests
per client IP per 60 seconds; request 81 returned 429 with `Retry-After: 57`.

No product defects remain. The only verification limitation was that this
container lacks Docker, so its image build command could not be run; the
locked release build and live deployment evidence passed instead.

## Prior builder handoff

Polish round 2 resolves all 25 cumulative findings from `review-1.md` and
`review-2.md`. The finding-by-finding record is `.factory/polish-2.md`.

## What changed

- Closed the landing-to-demo license race with request cancellation and a
  namespace-current guard before any storage write.
- Extended `@claim:demo-isolated` to reproduce the delayed live response while
  entering `/?demo=1`, then prove every real key remains unchanged.
- Removed the inaccurate public “Shared live” status.
- Added registered `first-supervisor-setup` and `shared-durable-storage`
  claims. Their Rust tests cover first setup, separate sessions, active and
  resolved holds, audit history, and a file-backed SQLite restart.
- Standardized the workspace name on **inventory holds**, corrected section
  headings, replaced the demo exit label, removed identity jargon, and made
  legal Return home controls real links.
- Injected the build SHA into the static 404 footer during the Vite build.
- Updated the claim manifest, demo guide, copy audit, README, catalog line,
  browser coverage, and release evidence without changing the blue-hour
  stockroom visual system.

## Verification

From a fresh clone at functional release commit
`e16a5610c97e0b19b036fd8f6d41125dbb22ee5c`:

- `npm ci` — pass, zero audit vulnerabilities.
- Every one of the 20 `.factory/claims.json` commands, run independently —
  pass.
- `npm test` — 3 Vitest, 9 Node contract, and 19 Rust tests pass.
- `npm run check` — Svelte/TypeScript clean; Clippy passes with warnings denied.
- `cargo fmt --all -- --check` — pass.
- `npm run build` — pass; `dist/` produced. Initial JS is 95.43 KB raw /
  34.06 KB gzip; CSS is 21.29 KB raw / 5.61 KB gzip.
- `npm run test:e2e` — 20/20 pass, covering full workflow, keyboard, dialogs,
  390 px, 200% text, route focus, metadata, privacy, offline, security headers,
  and Axe.
- Live `.factory/polish-2-live.spec.ts` — 4/4 pass, including the delayed
  license race and cold mobile/desktop visits.
- Worker URL verifier — 200 in 617 ms, correct title/lang/H1/main/alts, no
  unlabeled buttons, no console errors.
- Lighthouse mobile — Performance 99, Accessibility 100, Best Practices 100,
  SEO 100; LCP 1.6 s, TBT 40 ms, CLS 0, 174 KiB transferred.
- Health load smoke — 100 concurrent requests, 100 HTTP 200 responses in
  343 ms (291 requests/second observed).

## Deployment

The scoped fleet release created revision
`sf-inventory-promise-hold--0000033`. It reported HTTPS 200, one replica, one
`sf-inventory-promise-hold-data` Azure Files volume mounted at `/data`, and a
matching `/health` build SHA. No unrelated resource, secret, app setting,
database, storage, DNS zone, or service was read or changed.

Run another release from a clean tree with:

```sh
npm run deploy
```

Verify locally with:

```sh
npm ci
npm test
npm run check
cargo fmt --all -- --check
npm run build
npm run test:e2e
```

## Known gaps and next steps

No review finding or in-scope product defect remains. New Pro purchases stay
unavailable because the independently recorded hosted checkout is unavailable;
existing license restore remains tested and functional. No paid provider is
embedded in this product.
