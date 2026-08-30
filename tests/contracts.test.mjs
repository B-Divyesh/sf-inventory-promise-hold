import assert from 'node:assert/strict';
import { chmod, mkdtemp, readFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { promisify } from 'node:util';
import { execFile as execFileCallback } from 'node:child_process';
import test from 'node:test';

const execFile = promisify(execFileCallback);

test('container uses current stable Rust and declares durable /data', async () => {
  const dockerfile = await readFile(new URL('../Dockerfile', import.meta.url), 'utf8');
  assert.match(dockerfile, /^ARG BUILD_SHA=dev$/m);
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
  assert.equal(await readFile(log, 'utf8'), 'deploy /data inventory-promise-hold /work/repo Dockerfile 8080\nverify inventory-promise-hold\n');
});
