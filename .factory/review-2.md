# Stock Promise — adversarial first-read review 2

**Review date:** 2026-09-02 UTC  
**Live URL:** <https://inventory-promise-hold.sociobot.in>  
**Live build:** `f87062ac9983001b415577f4e70a88299f67b661`  
**Reviewed repository:** `3a092abcbb24ee68f9cfca0a30b9c069563bae87`  
**Verdict: FAIL**

The cold landing screen is clear and the sample is useful. All 18 registered
claim commands pass. The verdict is still FAIL because the one-click demo can
write real license state after the demo banner appears, several public claims
are absent from the claim manifest, and minor copy and route issues remain.

## Cold first read

### 390 px phone, before scrolling

My reading: this lets distributors and resellers place timed team holds on
scarce stock while coworkers take orders in parallel. I should select **Try it
with sample data** first.

The initial 390×844 view contains all of the evidence needed to answer the
three questions:

> “Hold scarce stock before it is promised twice.”
>
> “For distributors and resellers taking orders in parallel, Stock Promise
> shows a timed team hold before stock is promised.”
>
> “Try it with sample data” and “Open a sample stockroom.”

The three facts also fit before the bottom of the first view. The environmental
image begins below them.

### 1440 px desktop, before scrolling

The same job, audience, and first action are visible without scrolling. The
stockroom image supports the inventory context without carrying required text.
No blocking first-screen comprehension issue was found at either width.

## Findings

### F-2-1 — BLOCKING — entering the demo can write real license state

**Exact claims:** Demo banner: “Demo — sample data, nothing is saved.” README:
“It never reads or writes live workspace or license state.”

**Location:** `frontend/src/App.svelte` starts `checkLicense(false, demo)` on
mount. When the landing page starts that request with `demo === false`, moving
to `/?demo=1` does not cancel the pending live check. `checkLicense` later
writes the live verdict key in `frontend/src/license.ts`.

**Verification:** In a fresh live browser context, I preloaded only
`sb_license:inventory-promise-hold`, delayed the billing verification response,
opened `/`, and selected **Try it with sample data** before the response. While
the demo banner was visible, the delayed response wrote
`sb_license:inventory-promise-hold:verdict` to real `localStorage`. The current
`@claim:demo-isolated` test passes because it opens `/demo` directly and never
tests the required one-click transition from the landing page.

**Why this matters:** The visible isolation promise is false for a supported
entry path. Demo mode must not finish a live-state write after it begins.

**Concrete fix:** Abort or invalidate every live license check when entering
demo mode, and guard storage writes against the current namespace. Extend
`@claim:demo-isolated` to begin at `/` with hostile real state, hold a live
verification response open, click the sample action, release the response, and
assert that every real key remains byte-for-byte unchanged.

### F-2-2 — BLOCKING — “Shared live” is an unlisted and inaccurate status claim

**Exact quote/location:** Desktop landing header: “Shared live”.

**Verification:** `.factory/claims.json` has no shared-live or connection-state
claim. In `frontend/src/App.svelte`, the label is selected only from
`navigator.onLine`; the landing page does not authenticate a workspace or
confirm a successful stock API response before showing it.

**Why this matters:** A cold visitor can read this as confirmation that a team
workspace is connected and synchronized when the browser merely has network
access.

**Concrete fix:** Hide workspace status on the public landing and access
screens. In the authenticated product, derive the label from a successful
workspace response. If a shared-live promise remains, add it to
`.factory/claims.json` and test two sessions plus loss and recovery of the
backend connection.

### F-2-3 — BLOCKING — public persistence and shared-state claims are unlisted

**Exact quotes/location:** Privacy page:

> “Stock Promise stores the inventory, customer references, operator names,
> and hold notes that your team enters.”
>
> “The hosted service stores this operational data in its durable database.”
>
> “Inventory records, temporary holds, outcomes, and the audit record are
> stored so coworkers see the same stock.”

README:

> “Hosted operational data includes inventory, customer references, operator
> names, hold notes, outcomes, and the audit record.”

**Verification:** No entry in `.factory/claims.json` states or locates durable
server storage, the stored data categories, or shared visibility. There is an
untagged pool-restart unit test and deployment contract coverage, but the
claims contract requires a listed, tagged sandbox test for visitor-facing
promises.

**Why this matters:** Persistence and shared visibility are central reasons to
use this backend product. A visitor cannot tell which registered check proves
these statements.

**Concrete fix:** Add a `shared-durable-storage` claim listing the privacy page
and README. Its test should create inventory and a hold through one session,
observe them through a second session, restart a file-backed app, and confirm
the same records and audit history remain.

### F-2-4 — BLOCKING — setup and deployment guarantees in README are unlisted

**Exact quotes/location:** README:

> “The first supervisor creates the location.”
>
> “The container starts with only `PORT` set (default `8080`), stores SQLite at
> `/data/stock-promise.db`, and uses a single replica.”
>
> “Deployment keeps the SQLite database in `/data` and runs one app replica.”
>
> “The release command refuses a dirty tree, checks the mounted single-replica
> topology, and verifies `/health` returns the committed build SHA.”

**Verification:** These behaviors have no corresponding entries in
`.factory/claims.json`. Some are exercised by ordinary E2E or contract tests,
but those tests are not registered or tagged as the proof for these README
claims.

**Why this matters:** Operators can rely on these setup and data-safety
guarantees. The claim manifest is incomplete even though related tests exist.

**Concrete fix:** Register separate setup and deployment claims, attach exactly
one tagged test to each, and list every README location. Alternatively, rewrite
the deployment section as explicit required configuration instead of asserting
behavior the claim contract does not cover.

### F-2-5 — MINOR — the main workspace uses three vague names

**Exact quotes/locations:** Landing button: “Open the live desk”; demo `<h1>`:
“Promise desk”; demo tab: “Live desk”.

**Why this matters:** “Desk” does not name the inventory task, and changing
between “live desk” and “promise desk” makes one place sound like two. The demo
heading does not make sense when heard by itself.

**Concrete fix:** Use one concrete term. For example: button **Open inventory
holds**, demo heading **Manage sample inventory holds**, and tab **Inventory
holds**.

### F-2-6 — MINOR — a section heading does not cover its contents

**Exact quote/location:** Under “What Stock Promise does not do”, the landing
page says: “Supervisors choose when resolved customer references, notes, and
operator names are removed.”

**Why this matters:** The sentence describes a supported privacy control, not a
product limit. The heading is inaccurate for half of the section.

**Concrete fix:** Rename the section **Limits and data retention**, or move the
retention sentence into a separate **Data retention** section.

### F-2-7 — MINOR — the Pro heading does not name the features

**Exact quote/location:** Landing heading: “Optional Pro convenience”.

**Why this matters:** “Convenience” does not tell a reader what the section
contains and could describe any paid product.

**Concrete fix:** Use **Pro profiles and reminders**.

### F-2-8 — MINOR — a README heading depends on missing context

**Exact quote/location:** README heading: “Try it first”.

**Why this matters:** A heading list does not reveal what “it” is or that this
section opens isolated sample data.

**Concrete fix:** Use **Try the sample stockroom**.

### F-2-9 — MINOR — README reintroduces unexplained identity jargon

**Exact quote/location:** README, **Run locally**: “Production defaults to CIAM
with the shared Sociobot tenant.”

**Why this matters:** Review 1 removed “CIAM” from the access explanation, but
the same unexplained abbreviation remains later in the document.

**Concrete fix:** Use “Production uses the shared Sociobot customer sign-in
service by default.”

### F-2-10 — MINOR — the README opens with unexplained audit jargon

**Exact quote/location:** README first paragraph: “Supervisors maintain stock,
convert or release holds, review the append-only audit record, and export
outcomes.”

**Why this matters:** “Append-only” is implementation language before the
document explains the benefit.

**Concrete fix:** Use “Supervisors maintain stock, resolve holds, review a
record of past changes that cannot be edited, and export outcomes.”

### F-2-11 — MINOR — “Start for real” does not start the real workflow

**Exact quote/location:** Demo banner link: “Start for real”.

**Verification:** The link clears demo state and returns to the landing page.
The visitor must then select **Open the live desk** to begin sign-in.

**Why this matters:** The result does not match the action label.

**Concrete fix:** Either open the live access flow directly and label it **Open
live inventory**, or keep the current destination and label it **Leave demo**.

### F-2-12 — MINOR — legal-page navigation uses a button for a URL

**Exact quote/location:** Privacy and Terms: button “Return home”.

**Why this matters:** Home is a real destination. A button removes normal link
behavior such as opening in a new tab or copying the destination.

**Concrete fix:** Render an `<a href="/">Return home</a>` and retain the SPA
navigation handler.

### F-2-13 — MINOR — the 404 footer omits the required build identifier

**Exact quote/location:** Static 404 footer: “Built by Param Factory ·
AI-assisted image.” Other routes include `build f87062ac9983`.

**Why this matters:** The required consistent footer includes a version or
build ID on every route. The error route cannot be tied to the deployed build.

**Concrete fix:** Inject the same short build SHA into `404.html` during the
production build.

## Copy audit

Word counts treat whitespace-separated words and numbers as tokens. No landing
or README sentence exceeds 22 words, and no banned marketing adjective appears.
The flags below concern jargon, context, claim coverage, or action accuracy.

### Landing page sentences

| Words | Sentence | Flag |
| ---: | --- | --- |
| 3 | One shared location. | — |
| 8 | Hold scarce stock before it is promised twice. | — |
| 19 | For distributors and resellers taking orders in parallel, Stock Promise shows a timed team hold before stock is promised. | — |
| 4 | Open a sample stockroom. | — |
| 4 | Timed holds expire automatically. | — |
| 7 | The sample never changes a live stockroom. | — |
| 6 | New Pro purchases are temporarily unavailable. | — |
| 2 | List stock. | — |
| 8 | Add the SKUs that one location can promise. | — |
| 3 | Place a hold. | — |
| 7 | Staff name the customer, quantity, and expiry. | — |
| 2 | Resolve it. | — |
| 7 | A supervisor converts or releases the hold. | — |
| 16 | It is not a legal reservation, warehouse system, storefront, or replacement for your system of record. | — |
| 12 | Supervisors choose when resolved customer references, notes, and operator names are removed. | F-2-6 |
| 12 | A verified Pro license enables local operator profiles and on-device expiry reminders. | — |
| 9 | Core holds and CSV export do not require Pro. | — |
| 6 | New Pro purchases are temporarily unavailable. | — |
| 6 | Timed shared holds for one location. | — |
| 8 | Built by Param Factory · build f87062ac9983 · AI-assisted image. | — |

Landing headings and actions were also checked. **Try it with sample data** is
a result-naming action. **Open the live desk**, **Promise desk**, and **Live
desk** are flagged in F-2-5. **What Stock Promise does not do** is flagged in
F-2-6, and **Optional Pro convenience** is flagged in F-2-7.

### README sentences

| Words | Sentence | Flag |
| ---: | --- | --- |
| 16 | Stock Promise is a single-location hold desk for distributors and resellers who take orders in parallel. | — |
| 11 | Staff create a timed hold before scarce stock is promised twice. | — |
| 15 | Supervisors maintain stock, convert or release holds, review the append-only audit record, and export outcomes. | F-2-10 |
| 3 | Live product: `https://inventory-promise-hold.sociobot.in`. | — |
| 9 | Open `https://inventory-promise-hold.sociobot.in/?demo=1` or choose Try it with sample data. | — |
| 11 | The demo starts with three realistic SKUs and an active hold. | — |
| 11 | It is stored only under `demo:stock-promise:*` in the current browser session. | — |
| 10 | It never reads or writes live workspace or license state. | F-2-1 |
| 10 | Reset clears every demo key and restores the shipped sample. | — |
| 10 | The hosted live desk uses Sociobot Microsoft Entra External ID. | — |
| 8 | Sign-in roles set what each person can do: | — |
| 8 | `staff` can view live availability and create holds. | — |
| 20 | `supervisor` can also maintain inventory, resolve holds, manage retention, view the audit record, export CSV, and erase a whole location. | — |
| 6 | The first supervisor creates the location. | F-2-4 |
| 7 | Register `https://inventory-promise-hold.sociobot.in/auth/callback` as the sign-in return address. | — |
| 16 | Hosted operational data includes inventory, customer references, operator names, hold notes, outcomes, and the audit record. | F-2-3 |
| 21 | Supervisors choose 30–730 days before resolved customer references, notes, and operator names are removed, and can permanently erase the complete location. | — |
| 15 | Do not put payment, health, passwords, or other sensitive data into customer references or notes. | — |
| 4 | See `/privacy` and `/terms`. | — |
| 7 | A hold is an internal coordination signal. | — |
| 16 | It is not a legal reservation, sale, warehouse allocation, or replacement for a system of record. | — |
| 4 | Timed holds expire automatically. | — |
| 18 | If two staff members try to hold the same last units, only the first accepted hold protects stock. | — |
| 10 | The audit record keeps past changes and cannot be edited. | — |
| 14 | A supervisor can permanently erase the whole location when it is no longer needed. | — |
| 14 | A verified existing Pro license enables saved operator profiles and optional on-device expiry reminders. | — |
| 9 | Core holds and CSV export do not require Pro. | — |
| 15 | New Pro purchases are temporarily unavailable; the settings screen can still restore an existing license. | — |
| 8 | Requirements: Node.js 22+, npm, and current stable Rust. | — |
| 9 | `AUTH_MODE=local` is only for local development and test coverage. | — |
| 9 | Production defaults to CIAM with the shared Sociobot tenant. | F-2-9 |
| 18 | The container starts with only `PORT` set (default `8080`), stores SQLite at `/data/stock-promise.db`, and uses a single replica. | F-2-4 |
| 12 | Optional production overrides are `ENTRA_TENANT_ID`, `ENTRA_TENANT_SUBDOMAIN`, `ENTRA_CLIENT_ID`, `DATABASE_PATH`, `FRONTEND_DIR`, `BUILD_SHA`, and `RUST_LOG`. | — |
| 9 | Claims and their sandbox tests are listed in `.factory/claims.json`. | — |
| 21 | The browser suite covers the sample demo, CSV export, mobile layout, keyboard, accessibility, service-worker update, offline demo reload, and security headers. | — |
| 12 | Deployment keeps the SQLite database in `/data` and runs one app replica. | F-2-4 |
| 20 | The release command refuses a dirty tree, checks the mounted single-replica topology, and verifies `/health` returns the committed build SHA. | F-2-4 |

README headings checked were **Timed inventory holds for parallel orders**,
**Try it first**, **Access and data**, **Hold safety and optional Pro features**,
**Run locally**, **Verify**, **Deploy**, and **License**. Only **Try it first**
is flagged in F-2-8. Code blocks, environment-variable lists, and bare link
labels are not prose sentences; they were still checked for consistency.

## Demo and sandbox verification

The landing action entered `/?demo=1` in one click. At 390 px, the first demo
view already showed the Harbor Parts sample, 19 available units, three held
units, three named SKUs, and **Create hold**. The persistent banner, **Reset
demo**, and **Start for real** were visible. Reset restored three SKUs and one
active Northline Plumbing hold.

Direct `/demo` use with hostile live keys made only same-origin requests, made
no `/api/` request, and left all real keys unchanged. The sample also reloaded
offline after service-worker control. F-2-1 records the separate transition
race that occurs when a live license check starts on `/` before demo entry.

## Registered claims

I cloned the current repository to a fresh temporary directory, ran `npm ci`,
and executed every `test` command in `.factory/claims.json` separately.

| Claim | Result |
| --- | --- |
| `demo-isolated` | PASS |
| `demo-seed-reset` | PASS |
| `no-tracking` | PASS |
| `browser-storage` | PASS |
| `hosted-access` | PASS |
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

No registered command failed. F-2-1 shows that the passing isolation test does
not cover the required landing-to-demo race. F-2-2 through F-2-4 identify
visitor-facing promises not represented in the manifest.

## Earlier finding verification

I read `.factory/review-1.md`, `.factory/polish-1.md`, and the prior
`.factory/handoff.md`. Each Review 1 finding was checked on the live build and
in current code, not accepted from the polish status alone.

| Earlier finding | Current verification |
| --- | --- |
| F-1-1 | Fixed: landing says “Open a sample stockroom”; `demo-seed-reset` lists the landing hero and starts from home. |
| F-1-2 | Fixed: all three facts end within the live 390×844 first view; the full E2E assertion also passes. |
| F-1-3 | Fixed: the live 404 has the wordmark, Demo/Privacy navigation, legal footer, favicon, and apple-touch icon. F-2-13 is a narrower remaining footer-version issue. |
| F-1-4 | Fixed: static and SPA 404 headings say “Page not found”. |
| F-1-5 | Fixed: desktop demo status says “Sample data”; “Shared live” is absent in demo. |
| F-1-6 | Fixed: the demo has no **Lock supervisor** control. |
| F-1-7 | Fixed in visitor copy: README, UI, and Privacy consistently use “audit record”. Internal test descriptions still use event as a database noun, not a competing visitor term. |
| F-1-8 | Fixed: README heading is “Timed inventory holds for parallel orders”. |
| F-1-9 | Fixed at the cited location: README says “Sign-in roles set what each person can do”. F-2-9 records a separate remaining CIAM occurrence. |
| F-1-10 | Fixed: README calls the callback URL the “sign-in return address”. |
| F-1-11 | Fixed: README uses the proposed `/data` and one-replica wording. F-2-4 concerns missing claim registration, not the wording. |
| F-1-12 | Fixed: live Terms contains the proposed fair-use sentence. |

No Review 1 finding is repeated under its old ID.

## Structure, links, accessibility, and identity

Home, demo, Privacy, and Terms have route-specific titles, one `<h1>`, a main
landmark, descriptions, canonicals, social images, icons, and consistent
headers and legal links. Deep links, Back/Forward heading focus, reduced
motion, 200% text reflow, offline demo reload, and 44 px targets passed. A live
Playwright/Axe run found no serious or critical accessibility violation and no
unexpected console error on those routes.

All discovered same-origin links returned 200, except the deliberately tested
unknown route, which returned the designed 404. The blue-hour stockroom image,
amber hold signals, clipped panels, and Georgia/system type pairing match
`.factory/design.md` and do not present as a generic SaaS template. F-2-12 and
F-2-13 are the remaining route-shell issues.

Local gates also passed: `npm test`, `npm run check`,
`cargo fmt --all -- --check`, `npm run build`, and the full 20-test
`npm run test:e2e` suite.

## Missed leverage

No AI feature is implied by the core job. Timed shared stock coordination must
stay deterministic. CSV import, CSV outcome export, shared backend state, and
expiry handling are already present, so no additional import, export, sync, or
AI feature is recorded as a missed-leverage finding.

## What would make this perfect

Fix the demo namespace race first, remove or prove the “Shared live” status,
and register the persistence, shared-state, setup, and deployment claims. Then
apply the copy, navigation-semantic, and 404 footer fixes in F-2-5 through
F-2-13. Repeat the cold phone and desktop visit, every claim command, the
delayed-license transition test, request logging, offline reload, link crawl,
route-focus checks, and accessibility scan. A perfect round has zero findings.
