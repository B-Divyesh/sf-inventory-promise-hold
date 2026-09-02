# Stock Promise — adversarial review 3 handoff — FAIL

Review 3 is recorded in `.factory/review-3.md`. No product code was changed.
The review found two blocking claim-contract gaps and five minor copy or site
structure issues.

The blocking gaps are: the hosted Sociobot token-storage sentence is assigned
to a test that runs only local authentication, and the access-screen statement
that operational stock and customer references are private to the location is
not registered in `.factory/claims.json`.

Verification completed:

- Cold live captures at 390×844 and 1440×900.
- One-click live demo, realistic seed, create/reset, hostile real-state
  isolation, same-origin request log, no cookies, and offline reload.
- Every one of the 20 claim commands from a clean clone; all commands passed.
- `npm test`, `npm run check`, `cargo fmt --all -- --check`, `npm run build`,
  and `npm run test:e2e`; all passed.
- Live regression suite: 4/4 passed.
- Live Axe scan of home, demo, Privacy, Terms, and 404: zero violations.
- Metadata, canonical, h1/main, deep-link, Back/Forward focus, link crawl,
  security headers, and designed 404 checks.

Evidence screenshots are:

- `.factory/qa-artifacts/review-3-first-read-mobile.png`
- `.factory/qa-artifacts/review-3-first-read-desktop.png`
- `.factory/qa-artifacts/review-3-demo-mobile.png`

Next work should address F-3-1 through F-3-7 in order, then repeat the complete
review. The live product remains functional; this FAIL is caused first by
untested/unlisted privacy promises, not by a failed registered command.
