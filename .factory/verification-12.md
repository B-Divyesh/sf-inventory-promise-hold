# Independent verification 12 — PASS

Verified candidate: `26292139a5a935a48fcc9146b6c1dc4745868373`

Live URL: <https://inventory-promise-hold.sociobot.in>

Verification date: 2026-09-02 UTC

## Verdict

**PASS.** The repaired candidate passes every registered claim from the clean
checkout, all repository quality gates, the cold first-read/demo gate, and the
independent local and live checks. The live service reports and serves the exact
candidate.

Defect count: **0 blocker, 0 high, 0 medium, 1 low**.

The deployment-only/clean-build failure recorded in verification 11 does not
recur. The first claim command performed its cold Rust build before Playwright's
readiness timer and passed. The first `npm test` performed its cold release
build before the startup assertion and passed.

## Finding

### LOW — demo tab labels break inside words at 390 px

On the 390 px demo, the three equal-width tabs wrap `Inventory`, `Outcomes`, and
`settings` inside words. They remain visible, operable, at least 44 px tall, and
cause no horizontal overflow, so the core workflow is unaffected. The broken
labels reduce visual polish and quick scanning. Evidence:
[`verification-12-mobile-demo.png`](verification-evidence/verification-12-mobile-demo.png).

Suggested follow-up: prevent mid-word wrapping and let the labels wrap only at
spaces, or use a compact mobile tab treatment.

## Mandatory first checks

### Claims gate

`.factory/claims.json` exists with 22 entries. After `npm ci` in the clean
candidate checkout, every listed command was invoked separately and passed:

| Claim | Result |
| --- | --- |
| `demo-isolated` | PASS |
| `demo-seed-reset` | PASS |
| `no-tracking` | PASS |
| `browser-storage` | PASS |
| `hosted-token-storage` | PASS |
| `hosted-access` | PASS |
| `location-data-access` | PASS |
| `first-supervisor-setup` | PASS |
| `shared-durable-storage` | PASS |
| `csv-export` | PASS |
| `offline-demo` | PASS |
| `role-boundary` | PASS |
| `rate-limit` | PASS |
| `retention-redaction` | PASS |
| `automatic-expiry` | PASS |
| `contested-stock-protection` | PASS |
| `append-only-audit` | PASS |
| `location-erasure` | PASS |
| `pro-profiles-reminders` | PASS |
| `pro-license-restore` | PASS |
| `pro-checkout-status` | PASS |
| `core-features-no-pro` | PASS |

The live landing page and README were cross-checked against the manifest. No
material unlisted visitor claim was found.

### Cold first-read and demo gate

**PASS.** A cold visitor sees “Hold scarce stock before it is promised twice,”
the audience “distributors and resellers taking orders in parallel,” and the
first action **Try it with sample data** beside “Open a sample stockroom.” One
click opens `/?demo=1`, already populated with three SKUs and one active hold.
The persistent banner says “Demo — sample data, nothing is saved” and provides
Reset demo and Leave demo.

The successful cold page had no console error, page error, or failed request.
Evidence includes
[`screenshot-desktop.png`](verification-evidence/verify-url-12/screenshot-desktop.png)
and
[`screenshot-mobile.png`](verification-evidence/verify-url-12/screenshot-mobile.png).

## Clean checkout and production build

- Checkout began clean at the exact candidate SHA.
- `npm ci`: passed; 143 packages installed, 0 vulnerabilities.
- `npm test`: passed on its first clean run — 3 Vitest, 11 Node
  contract/startup, and 20 Rust tests.
- `npm run check`: passed — 0 Svelte/TypeScript diagnostics and Clippy passed
  with warnings denied.
- `cargo fmt --all -- --check`: passed.
- `VITE_BUILD_SHA=26292139a5a935a48fcc9146b6c1dc4745868373 npm run build`:
  passed and produced `dist/`.
- `npm run test:e2e:all`: passed — 21 general browser tests and 1 hosted-auth
  browser test.
- The locked release backend build passed as part of `npm test`.
- Docker/Podman is not installed in this worker, so final image assembly could
  not be repeated. The exact locked frontend and release-backend stages passed,
  and all Docker/runtime contract tests passed.

## Functional and backend verification

The live demo was exercised without touching hosted operational data:

- Quantity `0` was rejected with “Value must be greater than or equal to 1.”
- Quantity `999` was rejected with “Value must be less than or equal to 9.”
- Holding all 9 available valve units changed the SKU to **Fully held**.
- Releasing that hold restored 9 available units.
- A normal two-unit hold was created and converted.
- CSV export had the expected header and contained both released and converted
  outcomes.
- Reset demo restored exactly three SKUs and one active hold.

An independent candidate-stamped release binary used a fresh temporary SQLite
database. Empty inventory data and quantity zero returned clear 400 responses.
Two simultaneous two-unit holds against three units returned exactly one 201
and one 409. After process restart, the same session read the location, SKU,
one active hold, one remaining unit, and the append-only audit entries.
`/health` retained the full candidate SHA.

The full automated suite additionally covered setup, roles, automatic expiry,
retention redaction, full location erasure, audit protection, and release
startup with only `PORT`.

## Live backend, identity, and request allowance

- `/health`: HTTP 200, `status:"ok"`, build
  `26292139a5a935a48fcc9146b6c1dc4745868373`.
- `/api/auth/config`: `mode:"ciam"`.
- **Sign in with Sociobot** redirects only to
  `sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650`
  with client `25c704f4-465a-47af-80ab-2c489466b697` and this product's
  `/auth/callback`.
- Anonymous bootstrap and hold creation return 401 without exposing or changing
  operational data.
- Observed read allowance: 80 requests/client/60 seconds. Request 81 returned
  429 with `Retry-After: 56`.
- Observed write allowance: 20 requests/client/60 seconds. The first 20
  unauthenticated probes returned 401; request 21 returned 429 with
  `Retry-After: 59`.
- A separate 100-request concurrent read smoke using distinct client identities
  returned 100/100 HTTP 200. Health remained HTTP 200.

## Privacy, accessibility, routing, and PWA

- The complete live home → demo → create/release → create/convert → CSV → reset
  flow requested only the product origin, created no cookies, and produced no
  console errors, page errors, or failed requests.
- Desktop and 390 px checks covered home, demo, privacy, terms, and the designed
  404. Every page had `lang=en`, one H1, one main landmark, image alternatives,
  no horizontal overflow, and zero serious/critical Axe findings.
- At 390 px every visible interactive target was at least 44 px in both
  dimensions. Setting root text to 200% caused no horizontal overflow.
- The first Tab focused the skip link with a visible 3 px sand outline. Its next
  Tab lands on the first main action. Dialog focus starts inside the dialog;
  Escape closes it and restores the opener.
- Back/forward navigation restored each route and focused its H1. Reduced-motion
  preference was active.
- The service worker controlled `/demo`, updated to
  `/sw.js?v=26292139a5a935a48fcc9146b6c1dc4745868373`, used a candidate-versioned
  cache, and reloaded the populated demo offline with an offline notice.
- Every discovered same-origin link returned 200. The deliberately unknown URL
  returned the designed 404. `robots.txt` and `sitemap.xml` returned 200; the
  sitemap lists home, demo, privacy, and terms.
- The required fleet URL verifier passed in 645 ms with no console errors, one
  H1, one main landmark, no missing image alternatives, and no unlabeled
  buttons. Its JSON is
  [`verify.json`](verification-evidence/verify-url-12/verify.json).

Successful routes produced no console errors. Chromium reports the expected
failed-document message when directly loading the intentionally HTTP 404 page;
there is no application exception or failed subresource on that page.

## Headers, deployment match, and budgets

- HTML: `no-cache, must-revalidate`.
- Hashed JavaScript/CSS: `public, max-age=31536000, immutable`.
- `/sw.js`, `/api/*`, and `/health`: no-store/no-cache policies as appropriate.
- Responses include HSTS, `nosniff`, same-origin referrer policy, restrictive
  Permissions-Policy, and a response-header CSP with `frame-ancestors 'none'`.
- Live and local candidate SHA-256 hashes match for HTML, entry JS, shared JS,
  and CSS. Live asset names are also identical to the candidate build.
- Entry JS: 34.6 KB gzip; lazy shared JS: 69.8 KB gzip; CSS: 5.9 KB gzip;
  mobile hero: 15.4 KB. No font is downloaded.
- Fresh mobile Lighthouse: performance **93**, accessibility **100**, best
  practices **100**, SEO **100**; FCP/LCP 1.50 s, TBT 311 ms, CLS 0, total
  transfer 181,697 bytes. Lighthouse does not produce lab INP. Report:
  [`verification-12-lighthouse.json`](verification-evidence/verification-12-lighthouse.json).

## Scope and missed leverage

No infrastructure, production records, secrets, DNS, or unrelated resources
were read or changed. Live writes were confined to isolated demo storage;
server-side write checks were unauthenticated and could not mutate data. CSV
import/export already covers the obvious adjacent bulk workflow. Generative AI
would not improve deterministic, atomic stock protection and is correctly
absent.
