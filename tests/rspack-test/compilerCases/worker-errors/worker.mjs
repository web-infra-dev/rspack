import assert from 'node:assert/strict';
import { once } from 'node:events';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parentPort } from 'node:worker_threads';
import { rspack } from '@rspack/core';
import {
  closeCompiler,
  runCompiler,
} from '@rspack/test-tools/helper/lifecycle';
import { createFsFromVolume, Volume } from 'memfs';

let failHook = false;
let failWrite = false;
const context = fileURLToPath(new URL('.', import.meta.url));
const compiler = rspack({
  context,
  entry: 'data:text/javascript,export default 42',
  mode: 'none',
  cache: false,
  output: {
    path: path.join(context, 'dist'),
    compareBeforeEmit: false,
  },
  plugins: [
    {
      apply(compiler) {
        // Plugin taps exercise DynThreadsafeFunction.
        compiler.hooks.make.tapPromise('WorkerErrors', async () => {
          if (failHook) throw new Error('worker hook rejection');
        });
      },
    },
  ],
});
const fs = createFsFromVolume(new Volume());
const writeFile = fs.writeFile.bind(fs);
fs.writeFile = (...args) => {
  // Output filesystem callbacks exercise the typed ThreadsafeFunction.
  if (failWrite) {
    args.at(-1)(new Error('worker filesystem rejection'));
  } else {
    writeFile(...args);
  }
};
compiler.outputFileSystem = fs;

// Keep a listener installed while builds run: once() removes its own listener,
// and native callbacks alone do not keep the worker event loop alive.
parentPort.on('message', () => {});
try {
  assert.equal((await runCompiler(compiler)).hasErrors(), false);
  parentPort.postMessage('ready');
  const [command] = await once(parentPort, 'message');
  if (command === 'check') {
    failHook = true;
    await assert.rejects(runCompiler(compiler), /worker hook rejection/);
    failHook = false;
    failWrite = true;
    await assert.rejects(runCompiler(compiler), /worker filesystem rejection/);
    failWrite = false;
    assert.equal((await runCompiler(compiler)).hasErrors(), false);
    parentPort.postMessage('checked');
    assert.deepEqual(await once(parentPort, 'message'), ['close']);
  } else {
    assert.equal(command, 'close');
  }
} finally {
  await closeCompiler(compiler);
  parentPort.close();
}
