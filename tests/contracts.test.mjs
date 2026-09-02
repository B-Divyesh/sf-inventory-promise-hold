import assert from 'node:assert/strict';
import { chmod, mkdtemp, readFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';
import { execFile as execFileCallback } from 'node:child_process';
import test from 'node:test';

const execFile = promisify(execFileCallback);

test('container uses current stable Rust and declares durable /data', async () => {
  const dockerfile = await readFile(new URL('../Dockerfile', import.meta.url), 'utf8');
  assert.match(dockerfile, /^ARG BUILD_SHA=dev$/m);
  assert.match(dockerfile, /FROM node:22-alpine AS frontend\nARG BUILD_SHA\nENV VITE_BUILD_SHA=\$BUILD_SHA/);
  assert.match(dockerfile, /^FROM rust:1-slim AS backend$/m);
  assert.doesNotMatch(dockerfile, /^FROM rust:1\.[0-9]+/m);
  assert.match(dockerfile, /ENV FRONTEND_DIR=.* DATABASE_PATH=\/data\/stock-promise\.db BUILD_SHA=\$BUILD_SHA/);
  assert.match(dockerfile, /VOLUME \["\/data"\]/);
});

test('runtime uses a single SQLite writer compatible with the durable Azure Files mount', async () => {
  const main = await readFile(new URL('../src/main.rs', import.meta.url), 'utf8');
  const db = await readFile(new URL('../src/db.rs', import.meta.url), 'utf8');
  assert.match(main, /\.vfs\("unix-none"\)/);
  assert.match(main, /\.journal_mode\(SqliteJournalMode::Delete\)/);
  assert.match(main, /\.max_connections\(1\)/);
  assert.match(main, /prepare_schema/);
  assert.match(main, /recover_empty_database_path/);
  assert.doesNotMatch(main, /connect_lazy_with/);
  assert.match(db, /ALTER TABLE sessions ADD COLUMN role/);
});

test('topology verifier requires the work-order one-replica /data mount', async () => {
  const deployment = await readFile(new URL('../deploy/verify-persistent-data.sh', import.meta.url), 'utf8');
  assert.match(deployment, /storage_name="sf-\$\{slug\}-data"/);
  assert.match(deployment, /volume_name="data"/);
  assert.match(deployment, /minReplicas == 1/);
  assert.match(deployment, /maxReplicas == 1/);
  assert.match(deployment, /mountPath == "\/data"/);
});

async function fakeAz(template) {
  const directory = await mkdtemp(join(tmpdir(), 'stock-promise-az-'));
  const az = join(directory, 'az');
  await writeFile(az, `#!/usr/bin/env bash\nprintf '%s' "$FAKE_AZ_RESPONSE"\n`);
  await chmod(az, 0o755);
  return { ...process.env, PATH: `${directory}:${process.env.PATH}`, AZURE_SUBSCRIPTION_ID: '00000000-0000-0000-0000-000000000000', FAKE_AZ_RESPONSE: JSON.stringify(template) };
}

function targetTemplate({ maxReplicas = 1, mounted = true } = {}) {
  return { properties: {
    provisioningState: 'Succeeded', latestRevisionName: 'sf-inventory-promise-hold--0000002', latestReadyRevisionName: 'sf-inventory-promise-hold--0000002',
    template: { scale: { minReplicas: 1, maxReplicas }, volumes: mounted ? [{ name: 'data', storageType: 'AzureFile', storageName: 'sf-inventory-promise-hold-data' }] : [], containers: [{ name: 'app', volumeMounts: mounted ? [{ volumeName: 'data', mountPath: '/data' }] : [] }] },
  } };
}

test('topology verifier accepts exactly one ready mounted target', async () => {
  const env = await fakeAz(targetTemplate());
  const { stdout } = await execFile('deploy/verify-persistent-data.sh', [], { env });
  const result = JSON.parse(stdout);
  assert.equal(result.minReplicas, 1);
  assert.equal(result.maxReplicas, 1);
  assert.equal(result.mounts[0][0].mountPath, '/data');
});

test('topology verifier rejects the split-brain deployment', async () => {
  const env = await fakeAz(targetTemplate({ maxReplicas: 3, mounted: false }));
  await assert.rejects(execFile('deploy/verify-persistent-data.sh', [], { env }), (error) => {
    assert.equal(error.code, 1);
    assert.match(error.stderr, /Unsafe deployment/);
    return true;
  });
});

test('release delegates durable volume provisioning to the factory deployment command', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'stock-promise-release-'));
  const log = join(directory, 'calls');
  const deploy = join(directory, 'deploy');
  const verify = join(directory, 'verify');
  const curl = join(directory, 'curl');
  await writeFile(deploy, `#!/usr/bin/env bash\nprintf 'deploy %s %s\\n' "$WO_DATA_DIR" "$*" >> "$RELEASE_CALL_LOG"\n`);
  await writeFile(verify, `#!/usr/bin/env bash\nprintf 'verify %s\\n' "$*" >> "$RELEASE_CALL_LOG"\n`);
  await writeFile(curl, `#!/usr/bin/env bash\nprintf '{"build_sha":"%s","status":"ok"}' "$EXPECTED_RELEASE_SHA"\n`);
  await Promise.all([chmod(deploy, 0o755), chmod(verify, 0o755), chmod(curl, 0o755)]);
  const { stdout: head } = await execFile('git', ['rev-parse', 'HEAD']);
  await execFile('deploy/release.sh', [], { env: {
    ...process.env, PATH: `${directory}:${process.env.PATH}`, FACTORY_CONTAINER_DEPLOY_SCRIPT: deploy,
    PERSISTENT_DATA_VERIFY_SCRIPT: verify, RELEASE_CALL_LOG: log, EXPECTED_RELEASE_SHA: head.trim(), ALLOW_DIRTY_RELEASE: '1',
  } });
  const repository = fileURLToPath(new URL('..', import.meta.url)).replace(/\/$/, '');
  assert.equal(await readFile(log, 'utf8'), `deploy /data inventory-promise-hold ${repository} Dockerfile 8080\nverify inventory-promise-hold\n`);
});

test('recorded unavailable checkout is not offered by the product source', async () => {
  const [app, legal, license] = await Promise.all([
    readFile(new URL('../frontend/src/App.svelte', import.meta.url), 'utf8'),
    readFile(new URL('../frontend/src/Legal.svelte', import.meta.url), 'utf8'),
    readFile(new URL('../frontend/src/license.ts', import.meta.url), 'utf8'),
  ]);
  assert.match(app, /Paid upgrades are temporarily unavailable\./);
  assert.match(legal, /New purchases are temporarily unavailable\./);
  assert.doesNotMatch(`${app}\n${legal}\n${license}`, /\/checkout/);
});

test('every registered claim has one tagged sandbox test', async () => {
  const manifest = JSON.parse(await readFile(new URL('../.factory/claims.json', import.meta.url), 'utf8'));
  const source = (await Promise.all([
    '../tests/e2e/promise.spec.ts',
    '../tests/e2e/hosted-auth.spec.ts',
    '../src/api.rs',
    '../src/auth.rs',
    '../src/db.rs',
  ].map((path) => readFile(new URL(path, import.meta.url), 'utf8')))).join('\n');
  const ids = manifest.map((entry) => entry.id);
  assert.equal(new Set(ids).size, ids.length);
  for (const entry of manifest) {
    assert.equal(typeof entry.claim, 'string');
    assert.equal(typeof entry.where, 'string');
    assert.equal(typeof entry.test, 'string');
    assert.equal(typeof entry.sandbox, 'string');
    const tag = `@claim:${entry.id}`;
    assert.equal(source.split(tag).length - 1, 1, `${tag} must identify exactly one test`);
    assert.match(entry.test, /(?:npm run test:e2e(?::hosted)? -- --grep @claim:|cargo test claim_)/);
  }
});

test('reviewed visitor copy uses one plain term for each concept', async () => {
  const [readme, app, legal, notFound] = await Promise.all([
    readFile(new URL('../README.md', import.meta.url), 'utf8'),
    readFile(new URL('../frontend/src/App.svelte', import.meta.url), 'utf8'),
    readFile(new URL('../frontend/src/Legal.svelte', import.meta.url), 'utf8'),
    readFile(new URL('../frontend/public/404.html', import.meta.url), 'utf8'),
  ]);
  assert.match(readme, /^# Timed inventory holds for parallel orders$/m);
  assert.match(readme, /^## Try the sample stockroom$/m);
  assert.match(readme, /Sign-in roles\s+set what each person can do:/);
  assert.match(readme, /as the sign-in\s+return address\./);
  assert.match(readme, /Production\s+uses the shared Sociobot customer sign-in service by default\./);
  assert.doesNotMatch(`${readme}\n${app}\n${legal}`, /audit (?:trail|ledger|events?)/i);
  assert.doesNotMatch(readme, /CIAM|work-order deployment|SPA redirect URI|append-only audit record/);
  assert.match(legal, /Do not interfere with normal service use or present inaccurate stock availability to customers\./);
  assert.doesNotMatch(legal, /bypass access controls/);
  assert.match(app, /Open a sample stockroom\./);
  assert.match(app, /Open inventory holds/);
  assert.match(app, /Manage sample inventory holds/);
  assert.match(app, /Limits and data retention/);
  assert.match(app, /The sample opens offline after your first visit\./);
  assert.match(app, /Preview sample inventory holds/);
  assert.match(app, /Open sample stockroom/);
  assert.match(app, /Sign in to view this location’s stock and customer references\./);
  assert.doesNotMatch(app, /system of record/);
  assert.doesNotMatch(readme, /system of record|internal coordination signal/);
  assert.match(readme, /A hold tells coworkers that stock may be needed for an order\./);
  assert.match(readme, /It does not replace your inventory\s+or order system\./);
  assert.match(app, />Leave demo<\/a>/);
  assert.doesNotMatch(app, /Shared live|Open the live desk|Promise desk|Live desk|Optional Pro convenience|Start for real/);
  const landing = app.slice(app.indexOf('<main id="main" class="landing-page">'), app.indexOf('{:else if loading}'));
  assert.doesNotMatch(landing, /Pro profiles and reminders|Optional team convenience/);
  assert.match(legal, /<a class="text-button" href="\/"/);
  assert.match(notFound, /<h1>Page not found<\/h1>/);
  assert.match(notFound, /build __BUILD_SHA__/);
});
