import { Worker } from 'worker_threads';

async function receive(worker) {
  try {
    return await new Promise((resolve, reject) => {
      worker.on('message', resolve);
      worker.on('error', reject);
    });
  } finally {
    await worker.terminate();
  }
}

it('keeps worker federation demand separate from its parent', async () => {
  expect(__webpack_require__.federation).toBeUndefined();
  expect((await import('./local.js')).default).toBe('local value');
  expect(await receive(new Worker(new URL('./plain-worker.js', import.meta.url)))).toEqual({
    federation: false,
    value: 'local value',
  });
  expect(await receive(new Worker(new URL('./remote-worker.js', import.meta.url)))).toEqual({
    federation: true,
    value: 'remote value',
    child: { federation: false, value: 'local value' },
  });
  expect(await receive(new Worker(new URL('./shared-worker.js', import.meta.url)))).toEqual({
    federation: true,
    value: 'shared value',
  });
});
