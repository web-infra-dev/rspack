import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { rspack } from '@rspack/core';

export default ['webpack', 'rspack'].flatMap((runtimeMode) =>
  ['superseded-dependency', 'invalid-manifest'].map((scenario) => ({
    name: `${runtimeMode}-${scenario}`,
    description: 'settles a nested update while an accept callback is pending',
    options(context) {
      return {
        context: context.getDist('input'),
        entry: './index.js',
        mode: 'development',
        target: 'async-node',
        devtool: false,
        experiments: { runtimeMode },
        output: {
          path: context.getDist('output'),
          filename: 'main.cjs',
          chunkFilename: '[name].cjs',
          library: { type: 'commonjs2' },
        },
        plugins: [new rspack.HotModuleReplacementPlugin()],
      };
    },
    compiler(_context, compiler) {
      compiler.outputFileSystem = fs;
    },
    async build(context, compiler) {
      const input = context.getDist('input');
      const output = context.getDist('output');
      fs.mkdirSync(input, { recursive: true });
      const build = async (changes) => {
        const modifiedFiles = new Set();
        for (const [name, source] of Object.entries(changes)) {
          const file = path.join(input, name);
          fs.writeFileSync(file, source);
          modifiedFiles.add(file);
        }
        const stats = await new Promise((resolve, reject) =>
          compiler.run((err, stats) => (err ? reject(err) : resolve(stats)), {
            modifiedFiles,
          }),
        );
        assert.equal(stats.hasErrors(), false, stats.toString());
        return stats;
      };
      await build({
        'index.js': `
          import value from './dep';
          require('./trigger');
          export const hot = module.hot;
          export const values = [];
          export const probe = () => import('./probe');
          export let loaded;
          module.hot.accept('./dep', () => values.push(value));
          module.hot.accept('./trigger', () => loaded = import(/* webpackChunkName: "lazy" */ './lazy'));
        `,
        'dep.js': 'export default await Promise.resolve(0);',
        'trigger.js': 'module.exports = 0;',
        'lazy.js': 'export default 0;',
        'probe.js': 'import { values } from "./index"; export default values;',
      });
      const app = await createRequire(import.meta.url)(
        path.join(output, 'main.cjs'),
      );
      const invalidManifest = scenario === 'invalid-manifest';
      await build({
        'dep.js': invalidManifest
          ? 'export default await Promise.resolve(1);'
          : 'export default await new Promise(() => {});',
        'trigger.js': 'module.exports = 1;',
      });
      const latest = await build({
        'dep.js': 'export default await Promise.resolve(2);',
        'lazy.js': 'export default 2;',
      });
      if (invalidManifest) {
        const manifest = latest.compilation
          .getAssets()
          .find(({ name }) => name.endsWith('.hot-update.json'));
        fs.writeFileSync(path.join(output, manifest.name), '{');
        const manifestError = await app.hot.check(true).then(
          () => assert.fail('hot.check(true) accepted an invalid manifest'),
          (error) => error,
        );
        assert.equal(manifestError.name, 'SyntaxError');
        await assert.rejects(app.loaded, (error) => error === manifestError);
        assert.equal(app.hot.status(), 'fail');
      } else {
        await app.hot.check();
        const applying = app.hot.apply();
        assert.equal((await app.loaded).default, 2);
        let timeout;
        try {
          await Promise.race([
            applying,
            new Promise((_, reject) => {
              timeout = setTimeout(
                () => reject(new Error('apply still waits for the superseded dependency')),
                3000,
              );
            }),
          ]);
        } finally {
          clearTimeout(timeout);
        }
        assert.deepEqual(app.values, [2, 2]);
        assert.equal(app.hot.status(), 'idle');
        // A new importer must not inherit the discarded dependency's queue.
        assert.deepEqual((await app.probe()).default, [2, 2]);
      }
    },
  })),
);
