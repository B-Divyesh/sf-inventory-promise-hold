import assert from 'node:assert/strict';
import { chmod, mkdtemp, readFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { promisify } from 'node:util';
import { execFile as execFileCallback } from 'node:child_process';
import test from 'node:test';

const execFile = promisify(execFileCallback);

test('container bakes build identity and declares its durable data boundary', async () => {
  const dockerfile = await readFile(new URL('../Dockerfile', import.meta.url), 'utf8');
  assert.match(dockerfile, /^ARG BUILD_SHA=dev$/m);
  assert.match(dockerfile, /ENV FRONTEND_DIR=.* BUILD_SHA=\$BUILD_SHA/);
  assert.match(dockerfile, /VOLUME \["\/data"\]/);
});

test('runtime startup does not request an exclusive SQLite journal-mode change', async () => {
  const main = await readFile(new URL('../src/main.rs', import.meta.url), 'utf8');
  assert.doesNotMatch(main, /\.journal_mode\(/);
  assert.match(main, /prepare_schema/);
  assert.match(main, /connect_lazy_with/);
  assert.match(main, /from_secs\(15\)/);
});

test('deployment pins SQLite to one replica on an Azure Files mount', async () => {
  const deployment = await readFile(new URL('../deploy/ensure-persistent-data.sh', import.meta.url), 'utf8');
  assert.match(deployment, /minReplicas: 1, maxReplicas: 1/);
  assert.match(deployment, /storageType:"AzureFile"/);
  assert.match(deployment, /mountPath:"\/data"/);
  assert.match(deployment, /verify-persistent-data\.sh/);
  assert.match(deployment, /PERSISTENT_READY_ATTEMPTS:-120/);
});

async function makeFakeAz(template) {
  const directory = await mkdtemp(join(tmpdir(), 'stock-promise-az-'));
  const az = join(directory, 'az');
  await writeFile(az, `#!/usr/bin/env bash\nprintf '%s' "$FAKE_AZ_RESPONSE"\n`);
  await chmod(az, 0o755);
  return {
    env: {
      ...process.env,
      PATH: `${directory}:${process.env.PATH}`,
      AZURE_SUBSCRIPTION_ID: '00000000-0000-0000-0000-000000000000',
      FAKE_AZ_RESPONSE: JSON.stringify(template),
    },
  };
}

function deploymentTemplate({ maxReplicas = 1, volumes = true } = {}) {
  return {
    properties: {
      provisioningState: 'Succeeded',
      latestRevisionName: 'stock--0000002',
      latestReadyRevisionName: 'stock--0000002',
      template: {
        scale: { minReplicas: 1, maxReplicas },
        volumes: volumes
          ? [{ name: 'stock-promise-data', storageType: 'AzureFile', storageName: 'data-inventory-promise-hold' }]
          : null,
        containers: [{
          name: 'app',
          volumeMounts: volumes ? [{ volumeName: 'stock-promise-data', mountPath: '/data' }] : null,
        }],
      },
    },
  };
}

test('topology verifier accepts exactly one ready replica with durable /data', async () => {
  const options = await makeFakeAz(deploymentTemplate());
  const { stdout } = await execFile('deploy/verify-persistent-data.sh', [], options);
  const result = JSON.parse(stdout);
  assert.equal(result.minReplicas, 1);
  assert.equal(result.maxReplicas, 1);
  assert.equal(result.mounts[0][0].mountPath, '/data');
});

test('topology verifier rejects the split-brain deployment from QA-01', async () => {
  const options = await makeFakeAz(deploymentTemplate({ maxReplicas: 3, volumes: false }));
  await assert.rejects(
    execFile('deploy/verify-persistent-data.sh', [], options),
    (error) => {
      assert.equal(error.code, 1);
      assert.match(error.stderr, /Unsafe deployment/);
      assert.match(error.stderr, /"maxReplicas": 3/);
      return true;
    },
  );
});

test('release entry point cannot finish before persistence is applied and verified', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'stock-promise-release-'));
  const log = join(directory, 'calls');
  const makeCommand = async (name) => {
    const command = join(directory, name);
    await writeFile(command, `#!/usr/bin/env bash\nprintf '%s\\n' '${name}' >> "$RELEASE_CALL_LOG"\n`);
    await chmod(command, 0o755);
    return command;
  };
  const deploy = await makeCommand('deploy');
  const persist = await makeCommand('persist');
  const verify = await makeCommand('verify');
  const curl = join(directory, 'curl');
  await writeFile(curl, `#!/usr/bin/env bash\nprintf '{"build_sha":"%s","status":"ok"}' "$EXPECTED_RELEASE_SHA"\n`);
  await chmod(curl, 0o755);
  const { stdout: head } = await execFile('git', ['rev-parse', 'HEAD']);

  await execFile('deploy/release.sh', [], {
    env: {
      ...process.env,
      PATH: `${directory}:${process.env.PATH}`,
      FACTORY_CONTAINER_DEPLOY_SCRIPT: deploy,
      PERSISTENT_DATA_APPLY_SCRIPT: persist,
      PERSISTENT_DATA_VERIFY_SCRIPT: verify,
      RELEASE_CALL_LOG: log,
      EXPECTED_RELEASE_SHA: head.trim(),
      ALLOW_DIRTY_RELEASE: '1',
    },
  });

  assert.equal(await readFile(log, 'utf8'), 'deploy\npersist\nverify\n');
});
