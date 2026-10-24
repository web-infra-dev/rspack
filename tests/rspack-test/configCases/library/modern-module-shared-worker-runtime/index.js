import { Worker } from 'node:worker_threads';
import shared from './shared.cjs';

it('should register shared factories without executing worker startup in the parent', async () => {
  expect(shared.value).toBe(42);
  const worker = new Worker(new URL('./worker.js', import.meta.url), { execArgv: [] });
  try {
    const value = await new Promise((resolve, reject) => {
      worker.once('message', resolve);
      worker.once('error', reject);
      worker.once('exit', code => reject(new Error(`Worker exited before responding: ${code}`)));
    });
    expect(value).toBe(42);
  } finally {
    await worker.terminate();
  }
});
