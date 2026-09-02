# Stock Promise — independent verification 8: PASS

Candidate `f87062ac9983001b415577f4e70a88299f67b661` at
<https://inventory-promise-hold.sociobot.in> **PASSED** fresh independent QA
on 2026-09-02. The live `/health` build identity exactly matched the candidate.

All 18 registered claim tests, `npm test`, `npm run check`, formatting, the
production frontend and locked production backend builds, and the full
20-scenario browser suite passed. Fresh live Playwright verification passed
5/5 across desktop/mobile, one-click demo isolation/reset, keyboard, reduced
motion, serious/critical axe, privacy request logging, headers/cache, route
handling, PWA offline reload, and candidate identity.

Read rate limiting was observed at 80 requests/client/minute: request 81
returned `429` and `Retry-After: 59`. No release-blocking, high, medium, or low
severity defects were found. The earlier verification-7 failure applied to the
older `b4fe5c7` candidate, whose deployment identity was this candidate; it is
superseded by this exact-candidate verification.

How to verify: run `npm ci`, every command in `.factory/claims.json`,
`npm test`, `npm run check`, `cargo fmt --all -- --check`, `npm run build`, and
`npm run test:e2e`; then run
`BUILD_SHA=f87062ac9983001b415577f4e70a88299f67b661 cargo build --release --locked`.
For live verification, use <https://inventory-promise-hold.sociobot.in> and
confirm `/health` returns that SHA.

Full evidence is in `.factory/verification-8.md`. No product code,
infrastructure, DNS, storage, secrets, or other product resources were changed
by this verification; only this handoff and the verification report were added.
