import assert from 'node:assert/strict';
import { Worker } from 'node:worker_threads';

export default async function run() {
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
            received = message === 'ok';
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
}
