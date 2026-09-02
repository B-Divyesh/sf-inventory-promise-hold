# Stock Promise — adversarial first-read review 3

**Review date:** 2026-09-02 UTC  
**Live URL:** <https://inventory-promise-hold.sociobot.in>  
**Live build:** `fea95f207aaae2bfd98db15ea3c75c03760b8dab`  
**Reviewed repository:** `19f688f2f02ae93cc43cb33e4842bca530e6f9f6`  
**Verdict: FAIL**

The cold landing screen is clear, the demo is useful and isolated, all 20
registered claim commands pass, and the routing shell is sound. The verdict is
still FAIL because one registered privacy claim is not proved by its
assigned test and a second privacy statement is absent from the claim
manifest. Five minor copy and structure findings also remain. PASS requires
zero findings and no untested claim.

## Cold first read

### 390 px phone, before scrolling

My reading: this lets distributors and resellers place timed holds on scarce
stock while coworkers take orders in parallel. I should select **Try it with
sample data** first.

All three answers are supported in the initial 390×844 view:

> “Hold scarce stock before it is promised twice.”
>
> “For distributors and resellers taking orders in parallel, Stock Promise
> shows a timed team hold before stock is promised.”
>
> “Try it with sample data” and “Open a sample stockroom.”

The same answer was available at 1440×900. No blocking first-screen
comprehension failure was observed. Evidence:
`qa-artifacts/review-3-first-read-mobile.png` and
`qa-artifacts/review-3-first-read-desktop.png`.

## Findings

### F-3-1 — BLOCKING — the hosted-token privacy claim is not tested

**Exact quote/location:** Privacy → **What is stored**: “Sociobot sign-in
tokens stay in the current browser session.”

**Verification:** The manifest assigns this to `browser-storage`. Its command
passes, but `playwright.config.ts` starts the app with `AUTH_MODE=local`. The
tagged test creates and checks the app's local supervisor token. It never
initializes Microsoft Authentication Library, completes or mocks hosted
Sociobot sign-in, or observes a hosted token. `frontend/src/auth.ts` configures
Microsoft Authentication Library with `cacheLocation: 'sessionStorage'`, but a
source setting is not the required observable sandbox test.

**Why this matters:** A visitor can rely on this privacy statement when
deciding whether to sign in. The passing test proves a different authentication
mode, so this part of the registered claim remains untested.

**Concrete fix:** Split hosted sign-in storage into its own claim and tagged
browser test. Run the frontend in hosted-auth mode with the identity exchange
stubbed, complete the callback, then assert that its authentication cache is
present in `sessionStorage` and absent from `localStorage`. Keep the existing
test for the local supervisor session and preferences.

### F-3-2 — BLOCKING — the access screen makes an unlisted privacy claim

**Exact quote/location:** Select **Open inventory holds** on the live landing
page. The access screen says: “Operational stock and customer references are
private to this location.”

**Verification:** `.factory/claims.json` has no claim for unauthenticated or
wrong-role access to operational data. The ordinary end-to-end workflow checks
two anonymous requests, but that test is untagged and is not registered as
proof of this sentence.

**Why this matters:** This is a data-access promise, not descriptive copy. A
business may enter customer references based on it.

**Concrete fix:** Add a `location-data-access` claim at the access screen and a
single tagged test that verifies unauthenticated, wrong-tenant, and role-less
requests cannot read inventory, holds, outcomes, or audit data. Rewrite the
sentence as “Sign in to view this location’s stock and customer references” so
the user action and boundary are explicit.

### F-3-3 — MINOR — the first-screen facts omit offline behavior

**Exact location:** The three facts are “Timed holds expire automatically.”,
“The sample never changes a live stockroom.”, and “New Pro purchases are
temporarily unavailable.”

**Why this matters:** The required first-screen fact set covers privacy,
offline behavior, and price or purchase status. Privacy and purchase status
are present, but the tested offline capability appears only in the README.

**Concrete fix:** Add “The sample opens offline after your first visit.” to the
first-screen facts and add the landing hero to the `offline-demo` claim's
`where` field. Keep all facts within the 390×844 initial view.

### F-3-4 — MINOR — the landing page has no product preview

**Exact location:** After the first-screen stockroom artwork, the next section
is **How it works**. The landing page never shows inventory rows, an active
hold, or an outcome.

**Why this matters:** The required landing structure calls for the product or
a live preview directly after the first screen. Environmental artwork gives
the site a distinct identity, but it does not show what the working interface
looks like.

**Concrete fix:** Add a compact, read-only sample panel after the hero showing
the three sample SKUs, the Northline Plumbing hold, and its time remaining.
Link the panel to `/?demo=1` and reuse the real demo's rendering and data shape
so it cannot drift into a decorative mockup.

### F-3-5 — MINOR — the paid-tier section has no price

**Exact quote/location:** Landing → **Pro profiles and reminders**: “A verified
Pro license enables local operator profiles and on-device expiry reminders.”
It then says new purchases are temporarily unavailable, but gives no price or
billing period.

**Why this matters:** The site structure requires an exact price for a paid
tier. The current copy explains the features but leaves the commercial offer
undefined.

**Concrete fix:** If Pro remains a landing-page tier, show its exact recurring
price and billing period beside the unavailable status. If there is no current
sellable tier, remove the landing-page tier presentation and keep an **Existing
Pro licenses** explanation only in settings and the terms page.

### F-3-6 — MINOR — “system of record” is unexplained jargon

**Exact quotes/locations:** Landing → **Limits and data retention**: “It is not
a legal reservation, warehouse system, storefront, or replacement for your
system of record.” README → **Access and data**: “It is not a legal
reservation, sale, warehouse allocation, or replacement for a system of
record.”

**Why this matters:** A small reseller should not need enterprise software
vocabulary to understand the product boundary.

**Concrete fix:** Use “It does not replace your inventory or order system” in
both locations; retain the separate legal-reservation and warehouse/storefront
limits.

### F-3-7 — MINOR — the README defines a hold with abstract language

**Exact quote/location:** README → **Access and data**: “A hold is an internal
coordination signal.”

**Why this matters:** “Coordination signal” describes a concept rather than
what a coworker sees or does.

**Concrete fix:** Use “A hold tells coworkers that stock may be needed for an
order.”

## Copy audit

Counts use whitespace-separated words. Headings and action labels are included
as copy items; code blocks and navigation labels are not prose sentences. No
item exceeds 22 words, neither page uses a banned marketing adjective, and the
landing actions use result-naming verbs. The two jargon flags and the missing
first-screen fact are identified below.

### Landing page

| Words | Copy | Result |
| ---: | --- | --- |
| 3 | One shared location. | Pass |
| 8 | Hold scarce stock before it is promised twice. | Pass |
| 19 | For distributors and resellers taking orders in parallel, Stock Promise shows a timed team hold before stock is promised. | Pass |
| 5 | Try it with sample data. | Pass |
| 4 | Open a sample stockroom. | Pass |
| 4 | Timed holds expire automatically. | F-3-3: useful claim, but the required offline fact is absent |
| 7 | The sample never changes a live stockroom. | Pass |
| 6 | New Pro purchases are temporarily unavailable. | Pass |
| 3 | Open inventory holds. | Pass |
| 3 | How it works. | Pass |
| 2 | List stock. | Pass |
| 8 | Add the SKUs that one location can promise. | Pass |
| 3 | Place a hold. | Pass |
| 7 | Staff name the customer, quantity, and expiry. | Pass |
| 2 | Resolve it. | Pass in its ordered-step context |
| 7 | A supervisor converts or releases the hold. | Pass |
| 4 | Limits and data retention. | Pass |
| 16 | It is not a legal reservation, warehouse system, storefront, or replacement for your system of record. | F-3-6: jargon |
| 12 | Supervisors choose when resolved customer references, notes, and operator names are removed. | Pass |
| 4 | Pro profiles and reminders. | Pass |
| 12 | A verified Pro license enables local operator profiles and on-device expiry reminders. | Pass; pricing structure is F-3-5 |
| 9 | Core holds and CSV export do not require Pro. | Pass |
| 6 | New Pro purchases are temporarily unavailable. | Pass |
| 6 | Timed shared holds for one location. | Pass |
| 10 | Built by Param Factory · build fea95f207aaa · AI-assisted image. | Pass |

Landing average: 6.8 words per item; maximum: 19. **Try it with sample data**
and **Open inventory holds** both name their results.

### README

| Words | Copy | Result |
| ---: | --- | --- |
| 6 | Timed inventory holds for parallel orders. | Pass |
| 17 | Stock Promise is a single-location inventory hold workspace for distributors and resellers who take orders in parallel. | Pass |
| 11 | Staff create a timed hold before scarce stock is promised twice. | Pass |
| 18 | Supervisors maintain stock, resolve holds, review a record of past changes that cannot be edited, and export outcomes. | Pass |
| 3 | Live product: https://inventory-promise-hold.sociobot.in. | Pass |
| 4 | Try the sample stockroom. | Pass |
| 9 | Open https://inventory-promise-hold.sociobot.in/?demo=1 or choose Try it with sample data. | Pass |
| 11 | The demo starts with three realistic SKUs and an active hold. | Pass |
| 11 | It is stored only under demo:stock-promise:* in the current browser session. | Pass |
| 10 | It never reads or writes live workspace or license state. | Pass |
| 10 | Reset clears every demo key and restores the shipped sample. | Pass |
| 3 | Access and data. | Pass |
| 9 | Hosted inventory holds use Sociobot Microsoft Entra External ID. | Pass; provider name is necessary setup information |
| 8 | Sign-in roles set what each person can do. | Pass |
| 8 | staff can view live availability and create holds. | Pass |
| 20 | supervisor can also maintain inventory, resolve holds, manage retention, view the audit record, export CSV, and erase a whole location. | Pass |
| 6 | The first supervisor creates the location. | Pass |
| 7 | Register https://inventory-promise-hold.sociobot.in/auth/callback as the sign-in return address. | Pass |
| 16 | Hosted operational data includes inventory, customer references, operator names, hold notes, outcomes, and the audit record. | Pass |
| 21 | Supervisors choose 30–730 days before resolved customer references, notes, and operator names are removed, and can permanently erase the complete location. | Pass |
| 15 | Do not put payment, health, passwords, or other sensitive data into customer references or notes. | Pass |
| 4 | See /privacy and /terms. | Pass |
| 7 | A hold is an internal coordination signal. | F-3-7: abstract jargon |
| 16 | It is not a legal reservation, sale, warehouse allocation, or replacement for a system of record. | F-3-6: jargon |
| 6 | Hold safety and optional Pro features. | Pass |
| 4 | Timed holds expire automatically. | Pass |
| 18 | If two staff members try to hold the same last units, only the first accepted hold protects stock. | Pass |
| 10 | The audit record keeps past changes and cannot be edited. | Pass |
| 14 | A supervisor can permanently erase the whole location when it is no longer needed. | Pass |
| 14 | A verified existing Pro license enables saved operator profiles and optional on-device expiry reminders. | Pass |
| 9 | Core holds and CSV export do not require Pro. | Pass |
| 15 | New Pro purchases are temporarily unavailable; the settings screen can still restore an existing license. | Pass |
| 2 | Run locally. | Pass |
| 8 | Requirements: Node.js 22+, npm, and current stable Rust. | Pass |
| 9 | AUTH_MODE=local is only for local development and test coverage. | Pass |
| 10 | Production uses the shared Sociobot customer sign-in service by default. | Pass |
| 14 | For deployment, set PORT to the listening port and mount persistent storage at /data. | Pass |
| 9 | Run one app replica so SQLite has one writer. | Pass |
| 12 | Optional production overrides are ENTRA_TENANT_ID, ENTRA_TENANT_SUBDOMAIN, ENTRA_CLIENT_ID, DATABASE_PATH, FRONTEND_DIR, BUILD_SHA, and RUST_LOG. | Pass |
| 1 | Verify. | Pass in README task context |
| 9 | Claims and their sandbox tests are listed in .factory/claims.json. | Pass |
| 21 | The browser suite covers the sample demo, CSV export, mobile layout, keyboard, accessibility, service-worker update, offline demo reload, and security headers. | Pass |
| 1 | Deploy. | Pass in README task context |
| 7 | Run this command from a clean commit. | Pass |
| 11 | The fleet configuration must mount /data and keep one app replica. | Pass |
| 10 | After deployment, confirm that /health reports the commit you released. | Pass |
| 1 | License. | Pass |
| 1 | MIT. | Pass |

README average: 9.7 words per item; maximum: 21.

Terminology remains consistent: **hold**, **inventory holds**, **available
stock**, **staff**, **supervisor**, **location**, and **audit record** each name
one concept.

## Demo and sandbox verification

From a fresh 390×844 context, the landing action opened `/?demo=1` in one
click. The first demo view already showed Harbor Parts, three named SKUs, 19
available units, one Northline Plumbing hold, and hold actions. The persistent
banner showed **Demo — sample data, nothing is saved**, **Reset demo**, and
**Leave demo**.

Creating a temporary hold produced only
`demo:stock-promise:operator` and `demo:stock-promise:state` session keys.
`localStorage` remained empty. Reset removed both demo keys and restored three
SKUs and one active hold; the temporary customer disappeared. The request log
contained only six same-origin GET requests and no `/api/` request. The context
had no cookies. The delayed-license live regression test also confirmed every
preloaded real key stayed byte-for-byte unchanged while entering and using the
demo. Evidence: `qa-artifacts/review-3-demo-mobile.png`.

The sample reloaded after the browser was switched offline following its first
visit. The offline capability is functional; F-3-3 concerns its omission from
the first-screen facts.

## Registered claim results

I cloned commit `19f688f2f02ae93cc43cb33e4842bca530e6f9f6` to a fresh temporary
directory, ran `npm ci`, and executed every manifest command separately. All
20 commands returned zero. F-3-1 concerns what one passing test actually
exercises, and F-3-2 concerns a sentence with no manifest entry.

| Claim | Result | Evidence |
| --- | --- | --- |
| `demo-isolated` | PASS | Tagged Playwright test; live delayed-response regression also passed |
| `demo-seed-reset` | PASS | Tagged Playwright test; live create/reset check and screenshot |
| `no-tracking` | PASS | Tagged Playwright test; live request log was same-origin only with no cookies |
| `browser-storage` | PASS command, incomplete proof | Tagged Playwright test runs local auth only; see F-3-1 |
| `hosted-access` | PASS | Tagged Rust test |
| `first-supervisor-setup` | PASS | Tagged Rust test |
| `shared-durable-storage` | PASS | Tagged file-backed Rust test |
| `csv-export` | PASS | Tagged Playwright download/content test |
| `offline-demo` | PASS | Tagged isolated-context Playwright test and live offline reload |
| `role-boundary` | PASS | Tagged Rust test |
| `rate-limit` | PASS | Tagged Rust test; live endpoint returned 429 with `Retry-After` |
| `retention-redaction` | PASS | Tagged Rust test |
| `automatic-expiry` | PASS | Tagged Rust test |
| `contested-stock-protection` | PASS | Tagged concurrent Rust test |
| `append-only-audit` | PASS | Tagged Rust test |
| `location-erasure` | PASS | Tagged Rust test |
| `pro-profiles-reminders` | PASS | Tagged Playwright test |
| `pro-license-restore` | PASS | Tagged Playwright test |
| `pro-checkout-status` | PASS | Tagged Playwright test |
| `core-features-no-pro` | PASS | Tagged Playwright test |

## Earlier-finding verification

I read both earlier reviews, both polish reports, and the current handoff. Each
earlier finding was checked against the live site and current code.

| Earlier finding | Current confirmation |
| --- | --- |
| F-1-1 | Fixed: helper copy is “Open a sample stockroom”; `demo-seed-reset` names the landing hero and enters from home. |
| F-1-2 | Fixed: the facts end at y=533 in the live 390×844 view. |
| F-1-3 | Fixed: the live 404 has the common header, Demo/Privacy navigation, footer, icons, and original artwork. |
| F-1-4 | Fixed: the 404 h1 is “Page not found”. |
| F-1-5 | Fixed: demo status is “Sample data”; no “Shared live” label is rendered. |
| F-1-6 | Fixed: the demo has no **Lock supervisor** control. |
| F-1-7 | Fixed: visitor copy consistently uses **audit record**. |
| F-1-8 | Fixed: the README h1 names timed inventory holds and parallel orders. |
| F-1-9 | Fixed: README says sign-in roles set what each person can do. |
| F-1-10 | Fixed: README calls the callback URL the sign-in return address. |
| F-1-11 | Fixed: README plainly requires `/data` and one SQLite writer. |
| F-1-12 | Fixed: Terms contains the proposed fair-use sentence. |
| F-2-1 | Fixed: live delayed license verification is aborted/guarded on demo entry; hostile real keys remain unchanged. |
| F-2-2 | Fixed: “Shared live” is absent from public and demo screens. |
| F-2-3 | Fixed: `shared-durable-storage` is registered and its file-backed restart test passes. |
| F-2-4 | Fixed: first setup is registered; deployment statements are requirements and verification instructions. |
| F-2-5 | Fixed: landing, demo h1, and tab use **inventory holds**. |
| F-2-6 | Fixed: the section is **Limits and data retention**. |
| F-2-7 | Fixed: the section is **Pro profiles and reminders**. |
| F-2-8 | Fixed: README uses **Try the sample stockroom**. |
| F-2-9 | Fixed: README no longer uses “CIAM”. |
| F-2-10 | Fixed: README explains that the record of past changes cannot be edited. |
| F-2-11 | Fixed: the demo exit link is **Leave demo** and returns home. |
| F-2-12 | Fixed: legal-page **Return home** controls are links. |
| F-2-13 | Fixed: the live 404 footer includes `build fea95f207aaa`. |

No earlier finding regressed. F-3-1 and F-3-2 are separate gaps found by this
round's full claim review.

## Structure, links, accessibility, and identity

Home, both demo entry URLs, Privacy, Terms, and the designed 404 have
route-specific titles, one h1, one main landmark, descriptions, canonicals,
Open Graph/Twitter metadata, favicon and apple-touch icon. The live 404 returns
HTTP 404 and provides home and sample routes. Back/Forward moves focus to the
route h1 and announces it. All discovered first-party links returned 200;
the only 404 in the crawl was the intentionally missing page and its same-page
skip fragment.

The live Axe scan reported zero violations on home, demo, Privacy, Terms, and
404. Console logs were clean at both cold viewports. The blue-hour stockroom
art, amber hold marks, clipped panels, and Georgia/system type pairing match
`.factory/design.md` and are distinct from a generic SaaS template. F-3-4 and
F-3-5 are the remaining skeleton issues.

## Local verification

- Every one of the 20 claim commands from the clean clone: command passed.
- `npm test`: 3 Vitest, 9 Node contract, and 19 Rust tests passed.
- `npm run check`: Svelte/TypeScript and Clippy passed with no findings.
- `cargo fmt --all -- --check`: passed.
- `npm run build`: passed and produced `dist/`; initial app JavaScript is
  95.43 KB raw / 34.06 KB gzip.
- `npm run test:e2e`: 20/20 passed.
- Live polish regression suite: 4/4 passed.
- Live Axe scans: zero violations on all five checked routes.
- Live `/health`: build `fea95f207aaae2bfd98db15ea3c75c03760b8dab`.
  Repository changes after that build are documentation-only.

## Missed leverage

No AI feature is justified by the brief. Shared timed holds, conflict handling,
and expiry need deterministic behavior. CSV inventory import, CSV outcome
export, shared backend state, and automatic expiry already cover the obvious
import/export/sync needs. No decorative AI or embedded provider key was found.

## What would make this perfect

Add observable hosted-token storage coverage and register the access-screen
privacy boundary first. Then add the missing offline fact, a real product
preview, and an exact-price-or-existing-license treatment for Pro. Replace the
two jargon phrases. Re-run all 20 claim commands, the live demo isolation flow,
route crawl, route-focus checks, and Axe scan. A perfect round has no remaining
finding.
