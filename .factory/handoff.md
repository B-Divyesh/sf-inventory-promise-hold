# Stock Promise — verification handoff

Work order: `inventory-promise-hold-verify-2`
Candidate: `89db0bdb7520ce50b0a054f3a90a0ac64bb86c10`
Live URL: <https://inventory-promise-hold.sociobot.in>
Completed: 2026-08-28 UTC

## Verdict: FAIL

The candidate passes all clean local source, build, browser, accessibility,
privacy, PWA, concurrency, audit, and single-database persistence checks. The
live deployment is nevertheless unsafe for its core job: it has no persistent
volume, allows three replicas, and serves mutually inconsistent SQLite state.

Critical defect `QA-01` is documented with exact reproduction evidence in
`.factory/verification-2.md`. After a successful live setup and end-to-end
hold lifecycle, a 1,000-request load smoke started three replicas. The same
public endpoint then returned 81 uninitialized and 39 initialized responses;
a valid session succeeded on only 19 of 60 bootstrap requests. Azure inspection
confirmed `maxReplicas: 3`, `volumes: null`, and `volumeMounts: null`. This can
lose data and allow duplicate promises across replicas.

## What passed

- `npm ci`: 139 packages, 0 vulnerabilities.
- `npm test`: 1 Vitest + 3 Node contract + 9 Rust tests passed.
- `npm run check`: Svelte/TypeScript clean; strict Clippy passed.
- `cargo fmt --all -- --check`: passed.
- `npm run build`: passed; `dist/` produced.
- Locked release Rust build with the candidate SHA: passed.
- `npm run test:e2e`: 5/5 passed.
- Live Playwright after QA initialization: 5/5 passed on desktop and 390 px,
  including keyboard, axe, console/page errors, headers, PWA update/offline,
  lifecycle, and CSV.
- Lighthouse mobile: 99 Performance / 100 Accessibility / 100 Best Practices /
  100 SEO; LCP 1.5 s, TBT 90 ms, CLS 0.
- Factory URL verifier: passed in 598 ms with zero console errors.
- Live build identity: 30/30 `/health` samples matched the candidate SHA.

## Required next step

Run the checked-in durable deployment step after the generic deploy, restore
the intended database if appropriate, and prove that the live template has an
Azure Files `/data` mount with min/max replicas both set to one. Then restart
the revision and repeat authenticated load testing; every status/bootstrap
request must observe one location and one ledger.

Do not release until `QA-01` is fixed and independently reverified.

## Verification notes

No product code or infrastructure was modified. Only this handoff and
`.factory/verification-2.md` were added/updated. Docker was unavailable in the
worker image; the locked frontend/backend production stages passed separately,
and the live container reports the exact candidate SHA.
