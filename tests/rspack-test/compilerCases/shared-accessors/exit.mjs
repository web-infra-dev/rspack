import assert from 'node:assert/strict';
import { writeSync } from 'node:fs';
import path from 'node:path';
import {
  isMainThread,
  parentPort,
  Worker,
  workerData,
} from 'node:worker_threads';
import { rspack } from '@rspack/core';
import { createFsFromVolume, Volume } from 'memfs';
import {
  closeCompiler,
  runCompiler,
} from '@rspack/test-tools/helper/lifecycle';

const mode = process.argv[2];
const code = isMainThread ? Number(process.argv[3]) : workerData;

if (isMainThread && mode === 'worker') {
  const worker = new Worker(new URL(import.meta.url), { workerData: code });
  worker.on('error', (error) => {
    throw error;
  });
  worker.on('exit', (exitCode) => {
    process.exitCode = exitCode;
  });
} else {
  // Native build callbacks alone do not keep a worker alive.
  parentPort?.ref();
  const entry = 'data:text/javascript,export default 42';
  const compiler = rspack({
    mode: 'none',
    entry,
    output: { path: path.join(import.meta.dirname, 'dist') },
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());
  let accessed = false;
  compiler.hooks.compilation.tap('SharedAccessorsExit', (compilation) => {
    compilation.hooks.finishModules.tap('SharedAccessorsExit', (modules) => {
      const module = [...modules].find((module) => module.resource === entry);
      assert(module);
      // Populate Module, NormalModule and KnownBuildInfo accessor caches.
      assert.equal(
        typeof Object.getOwnPropertyDescriptor(module, 'context').get,
        'function',
      );
      assert.equal(module.matchResource, undefined);
      assert(
        Array.isArray(
          module.buildInfo[Symbol.for('rspack.buildInfo.fileDependencies')],
        ),
      );
      accessed = true;
    });
  });
  try {
    const stats = await runCompiler(compiler);
    assert.equal(stats.hasErrors(), false);
    assert.equal(accessed, true);
  } finally {
    await closeCompiler(compiler);
  }
  // A synchronous marker proves the build and close finished before forced exit.
  writeSync(1, 'compiler closed\n');
  if (!isMainThread || mode === 'exit') process.exit(code);
  process.exitCode = code;
}
