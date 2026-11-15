import assert from 'node:assert/strict';
import path from 'node:path';
import { createRequire } from 'node:module';
import { rspack, experiments } from '@rspack/core';
import { closeCompiler } from '@rspack/test-tools/helper/lifecycle';

const require = createRequire(import.meta.url);

export default async function watch(root, nativeWatcher, persistent) {
  const virtual = new experiments.VirtualModulesPlugin({
    'virtual/value.js': 'module.exports = 1;',
    'translations/en.js': 'module.exports = "hello";',
  });
  const compiler = rspack({
    context: import.meta.dirname,
    mode: 'development',
    target: 'node',
    entry: './entry.js',
    devtool: false,
    output: {
      path: path.join(root, 'dist'),
      filename: 'bundle.cjs',
      library: { type: 'commonjs2' },
    },
    experiments: { newCache: true, nativeWatcher },
    cache: persistent
      ? {
          type: 'persistent',
          storage: { type: 'filesystem', directory: path.join(root, 'cache') },
        }
      : false,
    plugins: [virtual],
  });
  let watching;
  let timer;
  let round = 0;
  let observed = -1;
  const values = [1, 2, 1];
  try {
    await new Promise((resolve, reject) => {
      watching = compiler.watch({ aggregateTimeout: 10 }, (error, stats) => {
        try {
          assert.ifError(error);
          assert.equal(stats.hasErrors(), false, stats.toString());
          const bundle = path.join(root, 'dist/bundle.cjs');
          delete require.cache[require.resolve(bundle)];
          assert.deepEqual(require(bundle), {
            value: values[round],
            translations: { './en.js': 'hello' },
          });
          // Watchers may report an initial filesystem event before our edit.
          if (observed === round) return;
          observed = round;
          if (round === values.length - 1) return resolve();
          timer = setTimeout(() => {
            round++;
            // Repeating a write must preserve the timestamp; a subsequent
            // different write must still invalidate it, including a reversion.
            virtual.writeModule(
              'virtual/value.js',
              `module.exports = ${values[round]};`,
            );
            virtual.writeModule(
              'virtual/value.js',
              `module.exports = ${values[round]};`,
            );
          }, 100);
        } catch (error) {
          reject(error);
        }
      });
    });
  } finally {
    clearTimeout(timer);
    if (watching)
      await new Promise((resolve, reject) =>
        watching.close((error) => (error ? reject(error) : resolve())),
      );
    await closeCompiler(compiler);
  }
}
