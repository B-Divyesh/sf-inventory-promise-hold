import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

test('container bakes build identity and declares its durable data boundary', async () => {
  const dockerfile = await readFile(new URL('../Dockerfile', import.meta.url), 'utf8');
  assert.match(dockerfile, /^ARG BUILD_SHA=dev$/m);
  assert.match(dockerfile, /ENV FRONTEND_DIR=.* BUILD_SHA=\$BUILD_SHA/);
  assert.match(dockerfile, /VOLUME \["\/data"\]/);
});

test('deployment pins SQLite to one replica on an Azure Files mount', async () => {
  const deployment = await readFile(new URL('../deploy/ensure-persistent-data.sh', import.meta.url), 'utf8');
  assert.match(deployment, /\.scale\.minReplicas = 1/);
  assert.match(deployment, /\.scale\.maxReplicas = 1/);
  assert.match(deployment, /storageType:"AzureFile"/);
  assert.match(deployment, /mountPath:"\/data"/);
});
