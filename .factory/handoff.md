# Stock Promise — independent verification 10 handoff

## Result

**FAIL** for candidate `ff809d81cef840ec4f4e13e6387018728c1d69f5` at
<https://inventory-promise-hold.sociobot.in>.

The live deployment is the tested candidate and the product works end to end,
but one mandatory backend runtime requirement remains open.

## Release-blocking defect

**MEDIUM — no default startup configuration log.** Starting the release binary
as an unprivileged user with only `PORT` succeeds and `/health` reports the
candidate SHA, but stdout/stderr remains empty. With `RUST_LOG=info`, the
expected JSON configuration line appears and identifies the default database,
migration, generated instance identity, and CIAM mode. The runtime contract
requires that line without an extra environment variable. Configure tracing to
default to INFO when `RUST_LOG` is absent, then rerun verification.

Defect count: **0 blocker, 0 high, 1 medium, 0 low**.

## What passed

- All 22 registered claim commands passed after `npm ci`.
- `npm test`, `npm run check`, `cargo fmt --all -- --check`, the candidate-
  stamped frontend build, and `cargo build --release --locked` passed.
- `npm run test:e2e:all` passed 21 normal and 1 hosted-auth browser tests.
- Cold first-read, one-click sample, normal/boundary/invalid/recovery flows,
  CSV, reset, mobile, keyboard, focus, 200% text, reduced motion, Axe, and PWA
  offline/update checks passed live.
- Privacy checks found only same-origin requests, no cookies, and no normal-page
  console or page errors.
- Live read limiting allowed 80 requests and returned 429 on request 81 with
  `Retry-After`; write limiting allowed 20 and returned 429 on request 21 with
  `Retry-After`.
- A 100-request concurrent read smoke completed in 393 ms with 100/100 HTTP 200
  responses; health remained 200 afterward.
- `/health` reports the exact candidate. The live generated frontend files
  byte-match the candidate production build.
- Fresh Lighthouse mobile scores: performance 92, accessibility 100, best
  practices 100, SEO 100; LCP 1.7 s and CLS 0.

Docker/Podman is unavailable in this verifier container, so an image build was
not rerun. Dockerfile contract tests and both exact production builds passed.

## Reproduce

```sh
npm ci
npm test
npm run check
cargo fmt --all -- --check
BUILD_SHA=ff809d81cef840ec4f4e13e6387018728c1d69f5 npm run build
BUILD_SHA=ff809d81cef840ec4f4e13e6387018728c1d69f5 cargo build --release --locked
npm run test:e2e:all
```

Start `target/release/stock-promise` in a writable temporary directory as an
unprivileged user with an empty environment plus `PORT`; its log is 0 bytes.
Repeat with `RUST_LOG=info` to see the currently suppressed configuration line.

Full evidence: [verification-10.md](verification-10.md).
