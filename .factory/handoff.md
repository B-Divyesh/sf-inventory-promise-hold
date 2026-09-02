# Stock Promise repair handoff

## Result

The release-blocking clean-clone timeout reported in verifier commit
`805f4ab8c2bc02f21beab654bc60dd9db28be916` is repaired. Compilation now runs
as an explicit setup phase before Playwright readiness and release-startup
assertion clocks begin. Product behavior and assertions are unchanged.

## Root cause and repair

- `npm run test:e2e` previously started `cargo run` inside Playwright's
  120-second `webServer` window. It now runs `cargo build --locked` first.
- `npm test` previously ran `cargo build --release --locked` inside a Node test
  with a 180-second timeout. It now builds first, and the test verifies and
  starts the existing release binary under only `PORT`.
- Hosted-auth browser tests use the same explicit debug setup.
- `tests/contracts.test.mjs` locks this ordering and proves compilation cannot
  move back inside either behavioral timeout.
- `README.md` explains the clean-clone setup behavior.

## Exact cold-cache evidence

Environment: Node `22.23.2`, npm `10.9.8`, Rust/Cargo `1.98.0`, Playwright
`1.58.2`. Each repaired command used an independent empty npm cache, empty
Cargo home, empty `target/`, and `CARGO_BUILD_JOBS=1` to represent the slower
verifier worker.

Original candidate reproduction:

- `npm run test:e2e -- --grep @claim:demo-isolated` failed with
  `Timed out waiting 120000ms from config.webServer` during compilation.
- `npm test` timed out the release-startup test at 180,000 ms and exited after
  316.7 seconds with 9 passed and 1 cancelled Node test.

Repaired candidate:

- After cold `npm ci`, the exact demo-isolation command passed in 163 seconds.
  The 2m34s debug compile completed before Playwright started; the behavioral
  test then passed in 3.5 seconds.
- After a second independent cold `npm ci`, `npm test` passed in 477 seconds.
  Its 5m18s release compile completed before Node's startup test began. Results:
  3 frontend tests, 11 Node contract/startup tests, and 20 Rust tests passed.

## Complete verification

- `npm ci`: passed from an empty npm cache; 143 packages, 0 vulnerabilities.
- `npm test`: passed from empty npm/Rust caches as detailed above.
- `npm run check`: 0 Svelte/TypeScript diagnostics; Clippy passed with warnings
  denied.
- `cargo fmt --all -- --check`: passed.
- `BUILD_SHA=repair-verification npm run build`: passed and produced `dist/`.
  Entry JS is 34.66 KB gzip, lazy shared JS is 70.16 KB gzip, and CSS is 5.92
  KB gzip.
- `npm run test:e2e:all`: 21 product browser tests and 1 hosted-auth browser
  test passed.
- All 22 commands in `.factory/claims.json` passed when invoked independently.
- Browser coverage includes desktop keyboard/focus and dialog restoration;
  390 px layout, 44 px targets, and 200% text; Axe serious/critical checks;
  same-origin privacy; no cookies or console errors; reduced motion; service
  worker update; offline demo reload; and History API focus announcements.
- A local release served `/`, `/demo`, `/privacy`, and `/terms` with 200 and an
  unknown route with the designed 404. HTML used `no-cache`; hashed assets used
  immutable caching; `/sw.js`, `/api/*`, and `/health` used their required
  no-cache/no-store policies. CSP and security headers were present.
- Local rate checks observed request 81 return 429 for reads and request 21
  return 429 for writes, both with `Retry-After: 59`.
- A 100-request concurrent local read smoke returned 100/100 HTTP 200 in 526
  ms. Health remained available and reported the supplied build identity.

## Deployment

Use the repository-scoped release path:

```sh
npm run deploy
```

It delegates storage provisioning to the fleet, verifies exactly one ready
replica with `sf-inventory-promise-hold-data` mounted at `/data`, and requires
the live `/health` build SHA to match the committed source. Final live identity
and URL evidence are recorded after rollout.

## Known gaps and next steps

No product gap remains from verification 11. Paid upgrades remain intentionally
unavailable because the registered checkout still returns 404; tested copy and
the absence of a checkout link prevent customers from reaching it.
