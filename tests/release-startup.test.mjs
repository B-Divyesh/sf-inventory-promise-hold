import assert from 'node:assert/strict';
import { access, chmod, mkdtemp } from 'node:fs/promises';
import { createServer } from 'node:net';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn } from 'node:child_process';
import test from 'node:test';

const repository = fileURLToPath(new URL('..', import.meta.url)).replace(/\/$/, '');
const startupRecord = /"message":"configuration ready \(PORT defaults to 8080; database and instance identity persist locally\)"/;

async function unusedPort() {
  const server = createServer();
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const { port } = server.address();
  await new Promise((resolve, reject) => server.close((error) => error ? reject(error) : resolve()));
  return port;
}

function waitForStartupRecord(child) {
  return new Promise((resolve, reject) => {
    let output = '';
    const timeout = setTimeout(() => fail(new Error(`release binary did not emit its startup configuration record:\n${output}`)), 15_000);
    const append = (chunk) => {
      output += chunk;
      if (startupRecord.test(output)) {
        succeed();
      }
    };
    const onClose = (code, signal) => fail(new Error(`release binary stopped before startup logging (code ${code}, signal ${signal}):\n${output}`));
    const onError = (error) => fail(error);
    const cleanup = () => {
      clearTimeout(timeout);
      child.stdout.off('data', append);
      child.stderr.off('data', append);
      child.off('close', onClose);
      child.off('error', onError);
    };
    const succeed = () => {
      cleanup();
      resolve(output);
    };
    const fail = (error) => {
      cleanup();
      reject(error);
    };

    child.stdout.on('data', append);
    child.stderr.on('data', append);
    child.once('close', onClose);
    child.once('error', onError);
  });
}

async function waitForExit(child) {
  if (child.exitCode !== null || child.signalCode !== null) {
    return;
  }
  await new Promise((resolve) => child.once('close', resolve));
}

test('release binary emits the configuration record with only PORT', { timeout: 180_000 }, async () => {
  const port = await unusedPort();
  const directory = await mkdtemp(join(tmpdir(), 'stock-promise-release-'));
  await chmod(directory, 0o777);
  const environment = { PORT: String(port) };
  assert.deepEqual(Object.keys(environment), ['PORT']);
  const binary = join(repository, 'target/release/stock-promise');
  await access(binary);
  const child = spawn(
    '/usr/bin/setpriv',
    ['--reuid=65534', '--regid=65534', '--clear-groups', binary],
    { cwd: directory, env: environment, stdio: ['ignore', 'pipe', 'pipe'] },
  );

  try {
    const output = await waitForStartupRecord(child);
    assert.match(output, /"level":"INFO"/);
    assert.match(output, /"database_source":"default"/);
    assert.match(output, /"schema":"migrated"/);
    assert.match(output, /"instance_identity":"generated"/);
    assert.match(output, /"auth_mode":"ciam"/);
  } finally {
    child.kill('SIGTERM');
    await waitForExit(child);
  }
});
