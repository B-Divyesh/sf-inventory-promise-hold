# Stock Promise

Stock Promise is a shared, single-location inventory hold desk for small
distributors and resellers. Staff can place a short, visible hold while an order
is being written, so the same scarce units are not promised twice. Supervisors
can convert a hold into a stock deduction, release it, review automatic expiry,
and export the complete outcome ledger.

Live product: <https://inventory-promise-hold.sociobot.in>

## What v1 does

- Creates holds with an atomic SQLite write transaction and a current stock
  check, even when several operators act at once.
- Expires holds on the server every 30 seconds and during relevant requests.
- Protects stock changes, conversions, releases, audit history, and export with
  a supervisor PIN. Sessions live only in the current browser tab.
- Records creates, changes, conversions, releases, and expiries in an
  append-only audit log.
- Imports a simple `sku,name,on_hand` CSV and exports every hold outcome.
- Works at 390 px, from a keyboard, and shows loading, empty, error, expiry,
  stale/offline, and conflict states.
- Keeps core safety, audit, and export free. A $39 one-time Pro license adds
  saved operator profiles and local five-minute browser notifications through
  the Sociobot billing engine.

A soft hold is an internal coordination signal. It is not a legal reservation,
sale, warehouse allocation, or replacement for the system of record.

## Run locally

Requirements: Node.js 22+, npm, and Rust 1.98+.

```sh
npm ci
npm run build
DATABASE_PATH=./stock-promise.db FRONTEND_DIR=dist cargo run
```

Open <http://localhost:8080>. On first run, name the location and create a
6–12 digit supervisor PIN. For frontend development, run the backend command
above in one terminal and `npm run dev` in another; Vite proxies API requests to
port 8080.

The production container needs no configuration and listens on `PORT` (default
8080). Its SQLite database is stored at `/data/stock-promise.db`. Mount `/data`
on persistent storage. Optional overrides are `DATABASE_PATH`, `FRONTEND_DIR`,
`BUILD_SHA`, and `RUST_LOG`; none is required.

## Test and verify

```sh
npm test          # Vitest plus Rust unit/transaction tests
npm run check     # Svelte/TypeScript checks plus strict Clippy
npm run test:e2e  # Playwright Chromium at a 390 px viewport
npm run build     # reproducible frontend output in dist/
docker build -t stock-promise .
docker run --rm -p 8080:8080 -v stock-promise-data:/data stock-promise
```

Playwright is pinned to 1.58.2. The E2E test covers first-run setup, adding
stock, creating a hold, conversion, the outcome ledger, mobile layout, semantic
landmarks, legal navigation, and browser console errors.

## API outline

- `GET /health` — health and build SHA
- `GET /api/bootstrap` — location, live availability, holds, outcomes
- `POST /api/setup`, `POST|DELETE /api/session` — first run and supervisor access
- `POST /api/inventory`, `POST /api/inventory/:id` — guarded stock writes
- `POST /api/holds` — atomic hold creation
- `POST /api/holds/:id/resolve` — guarded convert or release
- `GET /api/audit`, `GET /api/export.csv` — guarded records

All state-changing inputs are validated at the edge and all SQL is
parameterized. The service emits structured JSON logs, applies browser security
headers, shuts down gracefully, and contains no analytics or third-party
runtime scripts/fonts.

## Product and design notes

The researched scope is in `.factory/brief.json` when supplied by the factory.
The product-specific visual thesis, accessibility decisions, generated-image
prompt, review, and provenance are in [`.factory/design.md`](.factory/design.md).
Operational verification and known gaps are in
[`.factory/handoff.md`](.factory/handoff.md).

## License

MIT. See [LICENSE](LICENSE).
