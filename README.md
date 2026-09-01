# Stock Promise

Stock Promise is a single-location hold desk for distributors and resellers who
take orders in parallel. Staff create a timed hold before scarce stock is
promised twice. Supervisors maintain stock, convert or release holds, review
the append-only audit record, and export outcomes.

Live product: <https://inventory-promise-hold.sociobot.in>

## Try it first

Open <https://inventory-promise-hold.sociobot.in/demo> or choose **Try it with
sample data**. The demo starts with three realistic SKUs and a live hold. It is
stored only in `demo:stock-promise:state` in the current browser session; it
never writes to a live stockroom. Reset it from the banner at any time.

## Access and data

The hosted live desk uses Sociobot Microsoft Entra External ID. CIAM app roles
control the work boundary:

- `staff` can view live availability and create holds.
- `supervisor` can also maintain inventory, resolve holds, manage retention,
  view the audit record, export CSV, and erase a whole location.

The first CIAM supervisor creates the location. The SPA redirect URI must be
registered as `https://inventory-promise-hold.sociobot.in/auth/callback`.

Hosted operational data includes inventory, customer references, operator
names, hold notes, outcomes, and audit events. Supervisors choose 30–730 days
before resolved customer references, notes, and operator names are removed,
and can permanently erase the complete location. Do not put payment, health,
passwords, or other sensitive data into
customer references or notes. See [/privacy](https://inventory-promise-hold.sociobot.in/privacy)
and [/terms](https://inventory-promise-hold.sociobot.in/terms).

A hold is an internal coordination signal. It is not a legal reservation,
sale, warehouse allocation, or replacement for a system of record.

## Hold safety and optional Pro features

Timed holds expire automatically. If two staff members try to hold the same
last units, only the first accepted hold protects stock. The audit record is
append-only. A supervisor can permanently erase the whole location when it is
no longer needed.

A verified existing Pro license enables saved operator profiles and optional
on-device expiry reminders. Core holds and CSV export do not require Pro. New
Pro purchases are temporarily unavailable; the settings screen can still
restore an existing license.

## Run locally

Requirements: Node.js 22+, npm, and current stable Rust.

```sh
npm ci
npm run build
AUTH_MODE=local DATABASE_PATH=./stock-promise.db FRONTEND_DIR=dist cargo run
```

`AUTH_MODE=local` is only for local development and test coverage. Production
defaults to CIAM with the shared Sociobot tenant. The container starts with
only `PORT` set (default `8080`), stores SQLite at `/data/stock-promise.db`,
and uses a single replica. Optional production overrides are
`ENTRA_TENANT_ID`, `ENTRA_TENANT_SUBDOMAIN`, `ENTRA_CLIENT_ID`,
`DATABASE_PATH`, `FRONTEND_DIR`, `BUILD_SHA`, and `RUST_LOG`.

## Verify

```sh
npm ci
npm test
npm run check
cargo fmt --all -- --check
npm run build
npm run test:e2e
BUILD_SHA=local-verification cargo build --release --locked
```

Claims and their sandbox tests are listed in `.factory/claims.json`. The
browser suite covers the sample demo, CSV export, mobile layout, keyboard,
accessibility, service-worker update, offline demo reload, and security
headers.

## Deploy

```sh
npm run deploy
```

The work-order deployment configuration mounts durable `/data` and enforces
one replica for SQLite. The release command refuses a dirty tree, checks the
mounted single-replica topology, and verifies `/health` returns the committed
build SHA.

## License

[MIT](LICENSE)
