# Stock Promise — adversarial review 2 handoff

Review 2 completed on 2026-09-02 against live build
`f87062ac9983001b415577f4e70a88299f67b661` and repository commit
`3a092abcbb24ee68f9cfca0a30b9c069563bae87`. The verdict is **FAIL**.

The full report is `.factory/review-2.md`. It records four blocking findings:
a landing-to-demo license-check race can write real `localStorage` after the
demo banner appears; “Shared live” is not a verified workspace status; public
durability/shared-storage claims are absent from `.factory/claims.json`; and
README setup/deployment guarantees are also unlisted. Nine minor copy,
navigation-semantic, and 404-footer findings are included.

Verification performed:

- cold live visits at 390×844 and 1440×1000;
- one-click seeded demo, reset, hostile-storage, request-log, and delayed
  license-response checks;
- every one of the 18 registered claim commands from a fresh clone: all pass;
- live Playwright route/demo/offline/Axe suite: 5/5 pass;
- live internal-link crawl and designed 404 check;
- `npm test`, `npm run check`, `cargo fmt --all -- --check`, `npm run build`,
  and full `npm run test:e2e`: all pass (20/20 E2E scenarios).

No product code, infrastructure, DNS, storage, secrets, or external resources
were changed. Only the review and this required handoff were added or updated.
The next repair should start with F-2-1 and add its delayed response regression
case before addressing the remaining findings.
