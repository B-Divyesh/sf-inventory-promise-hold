# Stock Promise — polish round 1 handoff

**Result: repaired, pushed, deployed, and verified.**

- Work order: `inventory-promise-hold-polish-1`
- Failed released candidate: `4abf5cdb2918d114564c2ccc780c6aa2633c0ac8`
- Product repair commit: `b4fe5c74d419d83b35fe39514770a5ca3ecfc586`
- First verified repair revision: `sf-inventory-promise-hold--0000031`
- Live URL: <https://inventory-promise-hold.sociobot.in>

## What changed

The landing helper now says “Open a sample stockroom,” and the manifest maps
that promise to a real one-click claim test. The primary action opens the
isolated `/?demo=1` sample; `/demo` remains an equivalent deep link. Demo mode
shows its persistent banner, reset/start-real actions, “Sample data” desktop
status, and no fake supervisor lock action.

At 390×844 the job, audience, sample action, all three facts, and live action
fit before the image. The static and SPA 404 experiences now say “Page not
found” and use the same product header, navigation, footer, icons, metadata,
and blue-hour stockroom identity as the rest of the site.

README and legal copy now use **audit record** consistently and explain it in
plain words. The reviewed CIAM, SPA, work-order, and access-control jargon was
replaced with the requested user-facing wording. The catalog description is a
92-character verb-first sentence. Copy and claims now have regression
contracts, including a clean-clone-safe deployment assertion.

## How it was verified

Environment: Node 22.23.2, npm 10.9.8, Rust/Cargo 1.98.0, Playwright 1.58.2.

From a separate clean clone of the repair commit:

```sh
npm ci
# each of the 18 commands in .factory/claims.json, independently
npm test
npm run check
cargo fmt --all -- --check
npm run build
npm run test:e2e
BUILD_SHA=b4fe5c74d419d83b35fe39514770a5ca3ecfc586 cargo build --release --locked
bash -n deploy/*.sh
```

Results: 18/18 claim commands; 3 Vitest, 9 Node contract, 17 Rust, and 20/20
browser scenarios passed. The browser suite covers full hold lifecycle, CSV,
demo isolation/reset, mobile and 200% text, keyboard/dialog/history focus,
route titles/canonicals/404, security/cache headers, privacy request logging,
service-worker updates, and offline demo reload. Axe found no serious or
critical issue, including on the static 404.

The locked build produced `dist/`. Initial application JS is 95.02 KB raw /
33.98 KB gzip and CSS is 21.29 KB raw / 5.61 KB gzip.

## Live evidence

`npm run deploy` built and deployed only `sf-inventory-promise-hold`. The
release check reported revision `0000031`, min/max replicas 1, the existing
`sf-inventory-promise-hold-data` Azure Files volume mounted at `/data`, HTTPS
200, and `/health` equal to the repair commit.

The cold live repair suite passed 5/5 on home, `/?demo=1`, privacy, terms, a
real 404, and offline reload. It also observed `429` plus `Retry-After` after
the public request allowance. The worker URL verifier passed with a 593 ms
load and no console errors. Axe CLI reported 0 violations on four public
routes. Lighthouse mobile scored 99 Performance, 100 Accessibility, 100 Best
Practices, and 100 SEO; LCP was 1.65 s, TBT 98 ms, and CLS 0.

Finding-by-finding evidence and screenshots are in `.factory/polish-1.md` and
`.factory/qa-artifacts/`.

## Known gaps and next steps

No review finding or acceptance gap remains. New Pro purchases remain
intentionally unavailable, as the verified checkout route is not registered;
existing license restore continues to work. No unrelated application,
database, key vault, storage account, staging slot, DNS zone, or resource was
read or changed.
