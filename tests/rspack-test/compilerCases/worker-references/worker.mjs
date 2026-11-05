import assert from 'node:assert/strict';
import path from 'node:path';
import { once } from 'node:events';
import { parentPort } from 'node:worker_threads';
import { rspack } from '@rspack/core';
import { createFsFromVolume, Volume } from 'memfs';
import {
  closeCompiler,
  createGCTracker,
  runCompiler,
} from '@rspack/test-tools/helper/lifecycle';

const tracker = createGCTracker();
let labels;
let nextLabel = 0;
const compiler = rspack({
  context: import.meta.dirname,
  mode: 'none',
  cache: false,
  entry: Array.from(
    { length: 4 },
    (_, i) => `./loader.mjs!data:text/javascript,export default ${i}`,
  ),
  output: { path: path.join(import.meta.dirname, 'dist') },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('WorkerReferences', (compilation) => {
          compiler.webpack.NormalModule.getCompilationHooks(compilation).loader.tap(
            'WorkerReferences',
            (context) => {
              context.trackReference = (data) => {
                const label = String(nextLabel++);
                labels.push(label);
                tracker.track(data, label);
              };
            },
          );
          compilation.hooks.processAssets.tap('WorkerReferences', () => {
            // Materialize BindingCell wrappers, retained until rebuild/close.
            for (const asset of compilation.getAssets()) {
              assert.equal(typeof asset.info, 'object');
            }
          });
        });
      },
    },
  ],
});
compiler.outputFileSystem = createFsFromVolume(new Volume());

async function build() {
  labels = [];
  const stats = await runCompiler(compiler);
  assert.equal(stats.hasErrors(), false, stats.toString({ all: false, errors: true }));
  assert.equal(labels.length, 4);
  // Successful compilation alone would also pass if Drop silently leaked refs.
  for (const label of labels) await tracker.waitForCollection(label);
}

parentPort.on('message', () => {});
try {
  await build();
  parentPort.postMessage('ready');
  for (;;) {
    const [command] = await once(parentPort, 'message');
    if (command === 'close') break;
    assert.equal(command, 'build');
    await build();
    parentPort.postMessage('built');
  }
} finally {
  await closeCompiler(compiler);
  parentPort.close();
}
