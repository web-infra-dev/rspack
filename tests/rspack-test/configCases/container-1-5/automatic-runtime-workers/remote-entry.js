import { Worker } from 'worker_threads';

it('retains federation for a remote entry while its local Worker has no federation runtime', async () => {
  expect((await import('remote/value')).default).toBe('remote value');
  expect(__webpack_require__.federation.instance).toBeTruthy();

  const worker = new Worker(new URL('./plain-worker.js', import.meta.url), {
    name: 'remote-entry-local-worker',
  });
  try {
    const result = await new Promise((resolve, reject) => {
      worker.once('message', resolve);
      worker.once('error', reject);
    });
    expect(result).toEqual({
      federation: EXPECT_NAMED_FEDERATION,
      value: 'local value',
    });
  } finally {
    await worker.terminate();
  }
});
