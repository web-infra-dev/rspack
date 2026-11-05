import assert from 'node:assert/strict';
import { once } from 'node:events';
import { Worker } from 'node:worker_threads';

export default async function run() {
  const workers = [];
  const start = async () => {
    const worker = new Worker(new URL('./worker.mjs', import.meta.url), {
      execArgv: [],
    });
    workers.push(worker);
    assert.deepEqual(await once(worker, 'message'), ['ready']);
    return worker;
  };
  const stop = async (worker) => {
    const exited = once(worker, 'exit');
    worker.postMessage('close');
    assert.deepEqual(await exited, [0]);
  };

  try {
    // The first compiler initializes the error resolver. Keep a second worker
    // and its binding alive while the first worker tears down its Node env.
    const first = await start();
    const second = await start();
    await stop(first);

    const checked = once(second, 'message');
    second.postMessage('check');
    assert.deepEqual(await checked, ['checked']);
    await stop(second);
  } finally {
    await Promise.all(workers.map((worker) => worker.terminate()));
  }
}
