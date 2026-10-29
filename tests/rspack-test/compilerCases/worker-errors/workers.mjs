import { writeSync } from 'node:fs';
import { threadId } from 'node:worker_threads';
const trace = (stage) => writeSync(2, `TRACE ${threadId} ${stage}\n`);
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
    trace(`spawned ${worker.threadId}`);
    worker.on('exit', code => trace(`worker exited ${code}`));
    assert.deepEqual(await once(worker, 'message'), ['ready']);
    trace(`ready ${worker.threadId}`);
    return worker;
  };
  const stop = async (worker) => {
    const exited = once(worker, 'exit');
    trace(`closing ${worker.threadId}`);
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
    trace('checking survivor');
    second.postMessage('check');
    assert.deepEqual(await checked, ['checked']);
    await stop(second);
  } finally {
    trace('finally terminating remaining workers');
    await Promise.all(workers.map((worker) => worker.terminate()));
    trace('finally completed');
  }
}
