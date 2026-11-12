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
  expect(Boolean(__webpack_require__.federation)).toBe(EXPECT_PARENT_FEDERATION);
  expect((await import('./local.js')).default).toBe('local value');
  expect(await receive(new Worker(new URL('./plain-worker.js', import.meta.url), { name: 'plain-worker' }))).toEqual({
    federation: EXPECT_NAMED_FEDERATION,
    value: 'local value',
  });
  expect(await receive(new Worker(new URL('./plain-worker.js', import.meta.url)))).toEqual({
    federation: EXPECT_UNNAMED_FEDERATION,
    value: 'local value',
  });
  expect(await receive(new Worker(new URL(/* webpackChunkName: "comment-worker" */ './plain-worker.js', import.meta.url), { name: 'outside-scope' }))).toEqual({
    federation: EXPECT_NAMED_FEDERATION,
    value: 'local value',
  });
  expect(await receive(new Worker(new URL('./remote-worker.js', import.meta.url), { name: 'remote-worker' }))).toEqual({
    federation: true,
    value: 'remote value',
    child: { federation: EXPECT_NAMED_FEDERATION, value: 'local value' },
  });
  expect(await receive(new Worker(new URL('./shared-worker.js', import.meta.url), { name: 'shared-worker' }))).toEqual({
    federation: true,
    value: 'shared value',
  });
});
