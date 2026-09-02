# Stock Promise repair handoff

## Result

Repaired the only release-blocking finding from independent verification 10:
the release process now emits its required structured startup configuration
record when its environment contains only `PORT`.

## What changed

- `src/main.rs` now uses `RUST_LOG` when supplied and otherwise starts tracing
  at `info`. This keeps operator-selected filters intact while making the
  required configuration record visible in the factory's minimal runtime
  environment.
- `tests/release-startup.test.mjs` is a release-binary regression test. It
  builds `target/release/stock-promise`, starts it as UID/GID 65534 in a fresh
  writable directory through `setpriv`, and gives it an environment containing
  exactly one variable: `PORT`. It asserts the JSON `INFO` configuration record
  reports `database_source:"default"`, `schema:"migrated"`,
  `instance_identity:"generated"`, and `auth_mode:"ciam"`.
- `npm test` includes that release-binary test, so the runtime contract cannot
  silently regress behind unit-only coverage.

## Reproduction and regression evidence

Before the repair, a candidate-stamped release binary was started from a
writable temporary directory as UID 65534 with `env -i PORT=4190`. `/health`
served successfully and the captured combined stdout/stderr log was exactly
**0 bytes**, reproducing verification 10.

After the repair, `node --test tests/release-startup.test.mjs` passed. The test
uses an empty environment plus `PORT` and observed the required `INFO`
configuration record without `RUST_LOG` or any other extra variable.

## Verification run

All commands below passed in this repair workspace on 2026-09-02 UTC.

- `npm ci` — installed 143 packages; audit reported 0 vulnerabilities.
- `npm test` — 3 frontend tests, 10 Node contract/runtime tests (including the
  release-binary startup test), and 20 Rust tests passed.
- `npm run check` — 0 Svelte diagnostics; Clippy passed with `-D warnings`.
- `cargo fmt --all -- --check` — passed.
- `BUILD_SHA=local-repair npm run build` — passed; initial JavaScript was
  104.83 KB gzip and CSS was 5.92 KB gzip.
- `BUILD_SHA=local-repair cargo build --release --locked` — passed.
- `npm run test:e2e:all` — passed: 21 product browser tests and 1 hosted-auth
  test. These cover the desktop and 390 px layouts, keyboard-only navigation,
  visible focus, dialogs, Axe serious/critical findings, 200% text, reduced
  motion, offline reload/update, privacy request boundaries, direct routes,
  metadata, and response policies.
- The full runners exercised all 22 registered `.factory/claims.json` claims,
  including demo isolation/reset, privacy, access boundaries, durable storage,
  rate limiting, CSV export, automatic expiry, append-only audit, and free
  core functionality.

Docker/Podman is unavailable in this worker, so image assembly was not run
locally. The Dockerfile contract is covered by `tests/contracts.test.mjs`; the
factory release command builds the scoped container image and verifies the
durable one-replica `/data` topology.

## Deployment and live checks

`npm run deploy` completed from a clean committed source. The scoped release
script deployed `inventory-promise-hold`, verified its own exact live build
identity, and checked the durable topology. The observed deployed revision was
`sf-inventory-promise-hold--0000037`: exactly one ready replica, with the
factory-managed `sf-inventory-promise-hold-data` Azure Files volume mounted at
`/data`.

Post-deploy checks passed:

- `/health` returned `status:"ok"` with the exact committed build SHA and
  `Cache-Control: no-store`.
- Direct `HEAD /privacy` and `HEAD /terms` each returned 200 with
  `Cache-Control: no-cache, must-revalidate`.
- The hashed entry asset returned
  `Cache-Control: public, max-age=31536000, immutable`.
- Unauthenticated `GET /api/bootstrap` returned 401, preserving the staff
  access boundary.

## Known gaps / next steps

- No product behaviour is intentionally deferred.
- The retained independent verification reports remain in `.factory/` as
  historical evidence; `verification-10.md` describes the pre-repair failure.
