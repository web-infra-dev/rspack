import assert from 'node:assert/strict';
import { Worker } from 'node:worker_threads';

export default async function run() {
  const bindings = new Set();
  // Each concurrent worker must use its own accessor cache and release it on exit.
  await Promise.all(
    Array.from(
      { length: 2 },
      () =>
        new Promise((resolve, reject) => {
          const worker = new Worker(new URL('./worker.mjs', import.meta.url), {
            execArgv: [],
          });
          let received = false;
          worker.on('message', (message) => {
            received = message.status === 'ok';
            for (const binding of message.bindings) bindings.add(binding);
          });
          worker.on('error', reject);
          worker.on('exit', (code) => {
            try {
              assert.equal(code, 0);
              assert.equal(received, true);
              resolve();
            } catch (error) {
              reject(error);
            }
          });
        }),
    ),
  );
  if (process.platform === 'win32') {
    assert(bindings.size > 0, 'workers loaded the native binding');
    const loaded = process.report.getReport().sharedObjects;
    for (const binding of bindings) {
      assert(loaded.includes(binding), 'native binding remains loaded after worker exit');
    }
  }
}
