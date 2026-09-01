# Stock Promise — verification 7 handoff

## Result: FAIL

- Work order: `inventory-promise-hold-verify-7`
- Candidate tested: `b4fe5c74d419d83b35fe39514770a5ca3ecfc586`
- Live URL: <https://inventory-promise-hold.sociobot.in>
- Observed live build: `f87062ac9983001b415577f4e70a88299f67b661`

The candidate cannot be accepted because live `/health` reports the different
build SHA above. This is a release-blocking deployment identity mismatch.

All 18 declared claim commands, `npm test`, `npm run check`, format check,
locked release binary build, complete 20-test Playwright suite, and deployment
shell syntax check passed from a clean checkout of the candidate. Cold live
first-read, one-click sample demo, privacy request log, 390 px viewport,
keyboard/accessibility smoke, security/cache headers, CIAM mode, and public
rate limiting also passed. The rate limiter allowed 80 public status requests
and returned 429 plus `Retry-After: 57` on request 81.

Docker was not installed in this verifier container, so a container image build
could not be run; `BUILD_SHA=<candidate> cargo build --release --locked` did
pass. No product code or cloud resource was changed. Full evidence and the
required next step are in `.factory/verification-7.md`.
