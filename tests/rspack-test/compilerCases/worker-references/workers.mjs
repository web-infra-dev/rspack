import { writeSync } from 'node:fs';
import { threadId } from 'node:worker_threads';
const trace = (stage) => writeSync(2, `TRACE ${threadId} ${stage}\n`);
import assert from 'node:assert/strict';
import { once } from 'node:events';
import { Worker } from 'node:worker_threads';

export default async function run(mode) {
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
  const build = async (worker) => {
    const built = once(worker, 'message');
    trace(`rebuilding ${worker.threadId}`);
    worker.postMessage('build');
    assert.deepEqual(await built, ['built']);
  };
  const close = async (worker) => {
    const exited = once(worker, 'exit');
    trace(`closing ${worker.threadId}`);
    worker.postMessage('close');
    assert.deepEqual(await exited, [0]);
  };

  try {
    // A initializes the deletion channel first. B keeps the addon loaded while
    // A's env is destroyed, then drops new references on native build threads.
    const first = await start();
    const second = await start();
    await Promise.all([build(first), build(second)]);
    if (mode === 'terminate') {
      // Leave the native compiler and its BindingCell references alive.
      assert.equal(await first.terminate(), 1);
    } else {
      await close(first);
    }
    await build(second);

    // A fresh env must get a fresh channel even if Node reuses an env address.
    const third = await start();
    await Promise.all([build(second), build(third)]);
    await Promise.all([close(second), close(third)]);
  } finally {
    trace('finally terminating remaining workers');
    await Promise.all(workers.map((worker) => worker.terminate()));
    trace('finally completed');
  }
}
