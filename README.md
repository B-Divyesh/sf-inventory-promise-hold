# Timed inventory holds for parallel orders

Stock Promise is a single-location inventory hold workspace for distributors and resellers who
take orders in parallel. Staff create a timed hold before scarce stock is
promised twice. Supervisors maintain stock, resolve holds, review a record of
past changes that cannot be edited, and export outcomes.

Live product: <https://inventory-promise-hold.sociobot.in>

## Try the sample stockroom

Open <https://inventory-promise-hold.sociobot.in/?demo=1> or choose **Try it with
sample data**. The demo starts with three realistic SKUs and an active hold. It is
stored only under `demo:stock-promise:*` in the current browser session. It
never reads or writes live workspace or license state. Reset clears every demo
key and restores the shipped sample.

## Access and data

Hosted inventory holds use Sociobot Microsoft Entra External ID. Sign-in roles
set what each person can do:

- `staff` can view live availability and create holds.
- `supervisor` can also maintain inventory, resolve holds, manage retention,
  view the audit record, export CSV, and erase a whole location.

The first supervisor creates the location. Register
`https://inventory-promise-hold.sociobot.in/auth/callback` as the sign-in
return address.

Hosted operational data includes inventory, customer references, operator
names, hold notes, outcomes, and the audit record. Supervisors choose 30–730 days
before resolved customer references, notes, and operator names are removed,
and can permanently erase the complete location. Do not put payment, health,
passwords, or other sensitive data into
customer references or notes. See [/privacy](https://inventory-promise-hold.sociobot.in/privacy)
and [/terms](https://inventory-promise-hold.sociobot.in/terms).

A hold tells coworkers that stock may be needed for an order. It is not a legal
reservation, sale, or warehouse allocation. It does not replace your inventory
or order system.

## Hold safety

Timed holds expire automatically. If two staff members try to hold the same
last units, only the first accepted hold protects stock. The audit record keeps
past changes and cannot be edited. A supervisor can permanently erase the whole location when it is
no longer needed.

## Run locally

Requirements: Node.js 22+, npm, and current stable Rust.

```sh
npm ci
npm run build
AUTH_MODE=local DATABASE_PATH=./stock-promise.db FRONTEND_DIR=dist cargo run
```

`AUTH_MODE=local` is only for local development and test coverage. Production
uses the shared Sociobot customer sign-in service by default. For deployment,
set `PORT` to the listening port and mount persistent storage at `/data`. Run
one app replica so SQLite has one writer. Optional production overrides are
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

The test scripts build the Rust server before browser readiness and release
startup checks, so a clean clone does not spend an assertion timeout compiling.

Claims and their sandbox tests are listed in `.factory/claims.json`. The
browser suite covers the sample demo, CSV export, mobile layout, keyboard,
accessibility, service-worker update, offline demo reload, and security
headers.

## Deploy

```sh
npm run deploy
```

Run this command from a clean commit. The fleet configuration must mount
`/data` and keep one app replica. After deployment, confirm that `/health`
reports the commit you released.

## License

[MIT](LICENSE)
