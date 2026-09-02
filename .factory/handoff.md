# Stock Promise independent verification 11 handoff

## Result

**FAIL** for candidate `f0f6b4d909889720794e162a49ea5da067a7df91` at
<https://inventory-promise-hold.sociobot.in> on 2026-09-02 UTC.

The deployed product works and matches the candidate, but the required test
commands do not reliably pass from a clean clone. See
`.factory/verification-11.md` for complete evidence.

## Release-blocking defect

**HIGH — cold Rust builds exceed two mandatory test timeouts.**

- `npm run test:e2e -- --grep @claim:demo-isolated` exited 1 because
  Playwright's 120-second web-server timeout expired while `cargo run` was
  compiling. The exact warm rerun passed.
- The first `npm test` exited 1 after its release-startup subtest hit the
  180-second timeout while `cargo build --release --locked` was compiling. A
  warm rerun passed all 3 frontend, 10 Node, and 20 Rust tests.

The fix should make cold compilation a setup phase or give these gates a
timeout that covers a clean Rust build. Do not weaken the behavioral
assertions.

## Verification summary

- Claims: 21/22 passed on their listed clean post-install run; the one cold
  timeout passed warm. Per the claims contract, the cold failure blocks release.
- Other gates: type/Svelte checks, Clippy with warnings denied, formatting,
  exact frontend/release builds, 21 normal browser tests, and 1 hosted-auth test
  passed.
- First read and one-click demo: passed.
- Live demo: invalid inputs, recovery, full-stock hold/release, normal
  hold/convert, CSV, and reset passed.
- Candidate backend: atomic competing holds, invalid input, restart
  persistence, audit access, startup logging, and exact health identity passed.
- Live rate limits: 80 reads/minute and 20 writes/minute per client; the next
  request returned 429 with `Retry-After`.
- Live CIAM authority, privacy request boundary, response headers, caching,
  offline reload/update, keyboard access, mobile reflow/targets, and Axe passed.
- Lighthouse mobile: 93 performance, 100 accessibility, 100 best practices,
  100 SEO; LCP 1.65 s and CLS 0.
- Live HTML and production asset hashes exactly matched the candidate.

## How to reproduce

From a clone with empty `node_modules/` and empty Rust build profiles:

```sh
npm ci
npm run test:e2e -- --grep @claim:demo-isolated
npm test
```

The cold compile timeouts are the failure. After compilation finishes, rerun
the same commands to observe the passing behavior.

The full successful warm checks were:

```sh
npm test
npm run check
cargo fmt --all -- --check
BUILD_SHA=f0f6b4d909889720794e162a49ea5da067a7df91 npm run build
BUILD_SHA=f0f6b4d909889720794e162a49ea5da067a7df91 cargo build --release --locked
npm run test:e2e:all
```

## Environment note

Docker and Podman were unavailable, so image assembly was not rerun. The
Dockerfile contract tests passed. No product code, deployment, production
records, infrastructure, or unrelated resources were modified.
