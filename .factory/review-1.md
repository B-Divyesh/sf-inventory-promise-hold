# Stock Promise — first-read review 1

**Review date:** 2026-09-01 UTC
**Live URL:** <https://inventory-promise-hold.sociobot.in>
**Live build:** `4abf5cdb2918d114564c2ccc780c6aa2633c0ac8`
**Verdict: FAIL**

The product is clear enough for a cold visitor to identify the job, audience,
and first action. The demo and every registered claim check pass. The verdict
remains FAIL because the claim contract has one unlisted landing assertion and
there are remaining route-shell and copy findings below. A PASS requires zero
findings.

## Cold first read

### 390 px phone, before scrolling

My reading: this helps distributors and resellers who take simultaneous orders
put a timed hold on limited stock, so another worker does not promise it. I
would select **Try it with sample data** first.

This is supported by the visible text:

> “Hold scarce stock before it is promised twice.”
>
> “For distributors and resellers taking orders in parallel, Stock Promise
> shows a timed team hold before stock is promised.”
>
> “Try it with sample data” and “See a working stockroom immediately.”

The same result was confirmed at 1440 px desktop. No blocking five-second
understanding issue was observed.

## Findings

### F-1-1 — BLOCKING — the landing has an unlisted visitor-facing claim

**Location and quote:** Home hero, beside the primary action: “See a working
stockroom immediately.”

**Check:** The sentence promises an observable demo result and an unmeasured
time expectation. `.factory/claims.json` has no entry that lists the landing
hero as its location or states that result. The existing seed/reset browser
check proves related behaviour, but the manifest does not cover this sentence.

**Why this matters:** A visitor can rely on this sentence when choosing the
demo. The claims contract requires every such statement to have a registered,
tagged sandbox check.

**Concrete fix:** Change the sentence to “Open a sample stockroom.” Update the
existing `demo-seed-reset` claim to include the landing hero and explicitly say
that the demo opens with the sample stockroom; keep its tagged browser check
as the observable proof. Alternatively, remove the helper sentence.

### F-1-2 — MINOR — the three plain facts begin below a 390 px first view

**Location and quote:** Home hero at 390×844. The fact group begins at y=861:
“Timed holds expire automatically.”, “The sample never changes a live
stockroom.”, and “New Pro purchases are temporarily unavailable.”

**Why this matters:** The mandatory first-screen shape includes all three plain
facts. At the requested phone size, a visitor needs to scroll to reach them.

**Concrete fix:** Reduce the mobile hero image/vertical spacing or place the
fact group before the action so all three facts fit in the initial 844 px view.

### F-1-3 — MINOR — the static 404 page does not use the required site shell

**Location and quote:** Direct `/404` and an unknown URL return the designed
page headed “This page is not here.” The document has one `<main>` and a
“Return home” link, but no site header, navigation, footer, favicon, or
apple-touch icon.

**Why this matters:** The header and footer are not consistent on every route,
and the route omits required route metadata assets. A person who reaches this
page cannot select Demo, Privacy, or Terms without first returning home.

**Concrete fix:** Give `frontend/public/404.html` the same wordmark/header,
Demo and Privacy navigation, footer, favicon, and apple-touch icon as the
other routes. Keep the 404 status and the return-home action.

### F-1-4 — MINOR — the 404 heading does not name the page state plainly

**Location and quote:** `/404` `<h1>`: “This page is not here.”

**Why this matters:** A route heading should identify the page without relying
on mood language. “Page not found” is the established, immediately usable
description.

**Concrete fix:** Replace the heading with “Page not found”.

### F-1-5 — MINOR — the demo header describes isolated sample data as live

**Location and quote:** `/demo` header status: “Shared live”. The persistent
banner correctly says “Demo — sample data, nothing is saved.”

**Why this matters:** The two messages describe different states. A visitor
can reasonably read the header as confirmation that the demo is connected to
the shared workspace, which conflicts with the isolation notice.

**Concrete fix:** In demo mode, replace the status with “Sample data” and hide
the live connection indicator. Keep “Shared live” only for a signed-in live
workspace.

### F-1-6 — MINOR — a demo control names an action it does not perform

**Location and quote:** `/demo` header button: “Lock supervisor”. Selecting it
announces that the demo is already isolated instead of changing the supervisor
state.

**Why this matters:** Buttons need result-naming verbs. This control suggests
that a visitor can change the sample role, but it has no corresponding result.

**Concrete fix:** Remove the control in demo mode, or replace it with a
non-interactive “Sample supervisor access” status label.

### F-1-7 — MINOR — audit terminology changes for the same record

**Location and quotes:** README uses “append-only audit record”, “audit
events”, and “audit ledger”. The privacy page uses “append-only audit trail”
and “append-only audit ledger”.

**Why this matters:** A cold visitor cannot tell whether these are one record
or separate product features.

**Concrete fix:** Use **audit record** everywhere. Define it once in plain
words: “The audit record keeps past changes and cannot be edited.”

### F-1-8 — MINOR — the README heading is only the product name

**Location and quote:** README first heading: “Stock Promise”.

**Why this matters:** A heading heard without surrounding context does not
identify the job or user.

**Concrete fix:** Replace it with “Timed inventory holds for parallel orders”.

### F-1-9 — MINOR — README access wording uses unexplained identity jargon

**Location and quote:** README, **Access and data**: “CIAM app roles control
the work boundary:”.

**Why this matters:** “CIAM” and “work boundary” do not explain what a staff
member can do.

**Concrete fix:** Replace it with “Sign-in roles set what each person can do:”.

### F-1-10 — MINOR — README setup wording uses an unexplained web-platform term

**Location and quote:** “The SPA redirect URI must be registered as
`https://inventory-promise-hold.sociobot.in/auth/callback`.”

**Why this matters:** The instruction is useful to an administrator, but its
main technical term is unexplained.

**Concrete fix:** Replace it with “Register
`https://inventory-promise-hold.sociobot.in/auth/callback` as the sign-in
return address.”

### F-1-11 — MINOR — README deployment language uses factory/process jargon

**Location and quote:** “The work-order deployment configuration mounts durable
`/data` and enforces one replica for SQLite.”

**Why this matters:** “Work-order” and “durable” describe internal process
terms rather than the operating requirement.

**Concrete fix:** Replace it with “Deployment keeps the SQLite database in
`/data` and runs one app replica.”

### F-1-12 — MINOR — the Terms page contains a prohibited technical-security phrase

**Location:** Terms → **Fair use**, second sentence, in the access-control
clause.

**Why this matters:** The phrase is not needed to explain acceptable product
use and does not match the plain-language tone used elsewhere.

**Concrete fix:** Replace the whole sentence with “Do not interfere with
normal service use or present inaccurate stock availability to customers.”

## Demo and sandbox check

Selecting **Try it with sample data** from a fresh 390 px context opened
`/demo` in one click. The first view contained the populated Harbor Parts
sample: three named SKUs, 19 available units, one active hold for Northline
Plumbing, and the active-hold actions. The persistent banner was present with
**Reset demo** and **Start for real**.

Normal demo use made only same-origin requests and no API request. Reset was
available and restored the shipped sample. The tagged isolation and reset tests
also passed from this checkout. This review did not observe a live-data write
from demo mode.

## Claims check

All 18 manifest commands passed after `npm ci`:

| Claim group | Result |
| --- | --- |
| 10 browser claim commands | PASS |
| 8 Rust claim commands | PASS |
| Full Playwright suite, 19 scenarios | PASS |

No registered claim test failed. F-1-1 remains because the landing assertion
is not represented in the manifest.

## Copy audit

Word counts below use words and numbers as tokens. No landing or README
sentence exceeds 22 words. The flags in the findings section cover the
unlisted assertion, inconsistent terminology, headings, and unexplained
jargon.

### Landing page

| Words | Copy |
| ---: | --- |
| 3 | One shared location. |
| 8 | Hold scarce stock before it is promised twice. |
| 19 | For distributors and resellers taking orders in parallel, Stock Promise shows a timed team hold before stock is promised. |
| 5 | Try it with sample data. |
| 5 | See a working stockroom immediately. |
| 4 | Timed holds expire automatically. |
| 7 | The sample never changes a live stockroom. |
| 6 | New Pro purchases are temporarily unavailable. |
| 4 | Open the live desk. |
| 3 | How it works. |
| 2 | List stock. |
| 8 | Add the SKUs that one location can promise. |
| 3 | Place a hold. |
| 7 | Staff name the customer, quantity, and expiry. |
| 2 | Resolve it. |
| 7 | A supervisor converts or releases the hold. |
| 6 | What Stock Promise does not do. |
| 16 | It is not a legal reservation, warehouse system, storefront, or replacement for your system of record. |
| 12 | Supervisors choose when resolved customer references, notes, and operator names are removed. |
| 3 | Optional Pro convenience. |
| 12 | A verified Pro license enables local operator profiles and on-device expiry reminders. |
| 9 | Core holds and CSV export do not require Pro. |
| 6 | New Pro purchases are temporarily unavailable. |

### README

| Words | Copy |
| ---: | --- |
| 16 | Stock Promise is a single-location hold desk for distributors and resellers who take orders in parallel. |
| 11 | Staff create a timed hold before scarce stock is promised twice. |
| 15 | Supervisors maintain stock, convert or release holds, review the append-only audit record, and export outcomes. |
| 3 | Live product: inventory-promise-hold.sociobot.in. |
| 11 | Open inventory-promise-hold.sociobot.in/demo or choose Try it with sample data. |
| 11 | The demo starts with three realistic SKUs and a live hold. |
| 12 | It is stored only under demo:stock-promise:* in the current browser session. |
| 10 | It never reads or writes live workspace or license state. |
| 10 | Reset clears every demo key and restores the shipped sample. |
| 10 | The hosted live desk uses Sociobot Microsoft Entra External ID. |
| 7 | CIAM app roles control the work boundary. |
| 8 | Staff can view live availability and create holds. |
| 17 | Supervisors can also maintain inventory, resolve holds, manage retention, view the audit record, export CSV, and erase a whole location. |
| 7 | The first CIAM supervisor creates the location. |
| 14 | The SPA redirect URI must be registered as inventory-promise-hold.sociobot.in/auth/callback. |
| 15 | Hosted operational data includes inventory, customer references, operator names, hold notes, outcomes, and audit events. |
| 22 | Supervisors choose 30–730 days before resolved customer references, notes, and operator names are removed, and can permanently erase the complete location. |
| 15 | Do not put payment, health, passwords, or other sensitive data into customer references or notes. |
| 4 | See /privacy and /terms. |
| 7 | A hold is an internal coordination signal. |
| 16 | It is not a legal reservation, sale, warehouse allocation, or replacement for a system of record. |
| 4 | Timed holds expire automatically. |
| 18 | If two staff members try to hold the same last units, only the first accepted hold protects stock. |
| 5 | The audit record is append-only. |
| 14 | A supervisor can permanently erase the whole location when it is no longer needed. |
| 14 | A verified existing Pro license enables saved operator profiles and optional on-device expiry reminders. |
| 9 | Core holds and CSV export do not require Pro. |
| 15 | New Pro purchases are temporarily unavailable; the settings screen can still restore an existing license. |
| 9 | Requirements: Node.js 22+, npm, and current stable Rust. |
| 11 | AUTH_MODE=local is only for local development and test coverage. |
| 9 | Production defaults to CIAM with the shared Sociobot tenant. |
| 20 | The container starts with only PORT set, stores SQLite at /data/stock-promise.db, and uses a single replica. |
| 22 | Optional production overrides are ENTRA_TENANT_ID, ENTRA_TENANT_SUBDOMAIN, ENTRA_CLIENT_ID, DATABASE_PATH, FRONTEND_DIR, BUILD_SHA, and RUST_LOG. |
| 11 | Claims and their sandbox tests are listed in .factory/claims.json. |
| 21 | The browser suite covers the sample demo, CSV export, mobile layout, keyboard, accessibility, service-worker update, offline demo reload, and security headers. |
| 13 | The work-order deployment configuration mounts durable /data and enforces one replica for SQLite. |
| 20 | The release command refuses a dirty tree, checks the mounted single-replica topology, and verifies /health returns the committed build SHA. |

## Earlier-review history

I read `.factory/verification.md`, `verification-2.md` through
`verification-6.md`, and the previous handoff. The live `/health` build is the
same build verified in the final history report. The following earlier findings
are confirmed fixed in current code and current live behaviour:

| Earlier IDs | Current confirmation |
| --- | --- |
| First verification: persistence, staff boundary, build identity, rate allowance, legal routes, cache policy, target size, and headers | Current health reports the full build SHA; live legal routes return 200; response headers include CSP, HSTS, nosniff, referrer policy, and permissions policy; current code and claim tests cover the staff boundary and rate response. |
| QA-01; QA3-03 | Current deployment history and `deploy` contract tests require one replica and durable `/data`; the current live build identity matches verification 6. |
| QA3-01, QA3-02, QA3-04 through QA3-09 | Claims manifest exists; all 18 commands pass; `/demo` is seeded and isolated; current code includes role and rate checks, hosted sign-in configuration, route metadata, robots, sitemap, visual design, and factory footer. |
| QA4-01 through QA4-05 | The product now states new Pro purchases are unavailable and exposes no checkout action; claim coverage, mobile sizing, canonical updates, and the build footer are covered by current tests and live checks. |
| QA5-01 through QA5-03 | The demo uses `demo:stock-promise:*` session keys; the isolation, seed/reset, privacy, storage, and hosted-access claim commands pass. Current browser tests also cover dialog return focus and Back/Forward heading focus. |

No earlier finding above was observed again. F-1-1 through F-1-12 are new
findings from this full first-read round.

## Structure, accessibility, and visual check

Home, demo, privacy, and terms have route-specific titles, one `<h1>`, a main
landmark, descriptions, canonicals, social metadata, and no console errors.
Direct deep links worked, Back/Forward focus is covered by the passing browser
suite, internal landing links returned 200, and live Axe checks found no
serious or critical issues on those four routes. The visual system is distinct:
the blue-hour stockroom image, amber promise indicators, clipped surfaces, and
Georgia/system type pairing match `.factory/design.md`; it does not resemble a
generic product template. The 404 route is the exception recorded in F-1-3
and F-1-4.

## Missed leverage

No additional AI step is expected from this brief. The core job is shared,
timed stock coordination, and adding a model feature would not improve that
job. CSV import and outcome export are already present, while live shared
updates are part of the product itself.

## What would make this perfect

Apply the twelve concrete fixes above, especially the registered landing claim
and the complete 404 shell. Then repeat this full review with the copy audit,
all claim commands, cold 390 px visit, demo reset/isolation check, route crawl,
and accessibility check. A perfect round has no remaining findings.
