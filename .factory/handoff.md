# Stock Promise — verification 5 handoff

**Result: FAIL**

- Candidate: `42d1acecc06636936823c34e7b25b7906c8b7a91`
- Live URL: <https://inventory-promise-hold.sociobot.in>
- Checked: 2026-09-01 UTC
- Full report: `.factory/verification-5.md`

## Release blockers

1. **High — demo storage is not isolated.** The demo reads and writes the
   non-demo `stock-promise:operator` local-storage key. Reset leaves that value
   behind. A cached real license is also read on `/demo` and prepares a license
   verification request. This conflicts with the visible “nothing is saved”
   statement and the required demo namespace boundary.
2. **Major — focus continuity is incomplete.** Escape closes the hold dialog
   but does not return focus to its opener. Browser Back and Forward also leave
   focus on the document body, and route announcements are stale or absent.
3. **Major — claims coverage is incomplete.** All 14 listed claim commands pass,
   but the broader demo storage statement and several README/privacy promises
   do not have their own manifest entries and tagged sandbox tests.

## Confirmed working

- The first screen says what the product does, who it serves, and provides a
  one-click sample demo.
- `npm ci`, `npm test`, `npm run check`, formatting, script syntax, production
  frontend build, 14/14 Playwright checks, and the locked release backend build
  pass.
- The live build SHA is the candidate SHA; key live assets match local output
  byte-for-byte.
- Normal, invalid, boundary, release, conversion, CSV, concurrent hold, audit,
  and SQLite restart-persistence checks pass.
- Live read allowance is 80 requests per minute and write allowance is 20;
  the next request returns `429` with `Retry-After: 59`.
- Microsoft sign-in uses `sociobotcustomers.ciamlogin.com`.
- Desktop/mobile semantics, serious/critical Axe checks, touch targets, 200%
  text reflow, reduced motion, normal console/page errors, response headers,
  caching, service-worker update, offline demo reload, and link checks pass.
- Live Lighthouse mobile: Performance 98, Accessibility 100, Best Practices
  100, SEO 100; LCP 1.7 s, CLS 0, TBT 110 ms, 171 KiB transferred.

## Reproduce

```sh
npm ci
npm test
npm run check
cargo fmt --all -- --check
npm run build
npm run test:e2e
BUILD_SHA=42d1acecc06636936823c34e7b25b7906c8b7a91 cargo build --release --locked
```

Detailed commands, observations, and evidence paths are recorded in
`.factory/verification-5.md` and `.factory/qa-artifacts/`.

No product code was changed. No deployment, infrastructure, database,
key-vault, or unrelated service was read, modified, or restarted.
