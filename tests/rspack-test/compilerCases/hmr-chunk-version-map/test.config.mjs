import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import vm from 'node:vm';
import { rspack } from '@rspack/core';

export default ['webpack', 'rspack'].map((runtimeMode) => ({
  name: runtimeMode,
  description:
    'patches chunk versions only during apply and preserves unchanged assets',
  compiler(_context, compiler) {
    compiler.outputFileSystem = fs;
  },
  options(context) {
    return {
      context: context.getDist('input'),
      entry: './index.js',
      mode: 'development',
      devtool: false,
      target: 'node',
      experiments: { runtimeMode },
      output: {
        path: context.getDist('output'),
        filename: '[name].js',
        chunkFilename: '[name].js',
        publicPath: '/',
      },
      optimization: {
        moduleIds: 'named',
        chunkIds: 'named',
        splitChunks: false,
      },
      plugins: [new rspack.HotModuleReplacementPlugin()],
    };
  },
  async build(context, compiler) {
    const input = context.getDist('input');
    fs.mkdirSync(input, { recursive: true });
    const write = (name, text) =>
      fs.writeFileSync(path.join(input, name), text);
    write('unchanged.js', 'module.exports = 10;');
    write('extra.js', 'module.exports = 20;');
    const output = context.getDist('output');
    const stateKey = `__chunkVersionMapTest_${runtimeMode}`;
    const state = (globalThis[stateKey] = {});
    let assets, versions, oldPage, unchangedSource, oldVersions, oldHash;
    try {
      for (let step = 0; step < 4; step++) {
        write(
          'index.js',
          `
        globalThis[${JSON.stringify(stateKey)}].step = ${step};
        globalThis[${JSON.stringify(stateKey)}].api = { hot: module.hot,
          load: () => import(/* webpackChunkName: "page" */ './page'),
          unchanged: () => import(/* webpackChunkName: "unchanged" */ './unchanged')
          ${step === 1 ? ', extra: () => import(/* webpackChunkName: "extra" */ "./extra")' : ''}
        };
        module.hot.accept();
      `,
        );
        write(
          'page.js',
          `module.exports = ${step ? 2 : 1}; module.hot.accept();`,
        );
        const stats = await new Promise((resolve, reject) =>
          compiler.run(
            (error, stats) => (error ? reject(error) : resolve(stats)),
            {
              modifiedFiles: new Set(
                ['index.js', 'page.js'].map((name) => path.join(input, name)),
              ),
            },
          ),
        );
        assert.equal(stats.hasErrors(), false, stats.toString());
        assets = Object.fromEntries(
          stats.compilation
            .getAssets()
            .map((asset) => [asset.name, asset.source.source().toString()]),
        );
        if (!step) {
          // Observe the internal map without making it a public runtime property.
          const main = assets['main.js'].replace(
            /(\S+) = Object.create\(null\);(\s*\[\[)/,
            `globalThis[${JSON.stringify(stateKey)}].versionMap = $1 = Object.create(null);$2`,
          );
          assert.notEqual(main, assets['main.js']);
          vm.runInThisContext(`(function(require, __dirname) { ${main}\n})`)(
            createRequire(path.join(output, 'main.js')),
            output,
          );
          versions = state.versionMap;
          oldPage = versions.page;
          unchangedSource = assets['unchanged.js'];
        } else {
          const { hot } = state.api;
          const updates = Object.entries(assets).filter(([name]) =>
            name.endsWith('.hot-update.js'),
          );
          const currentUpdates = updates.filter(([name]) =>
            name.includes(oldHash),
          );
          assert(
            !currentUpdates.some(([, source]) =>
              source.includes(versions.unchanged),
            ),
            'patch must omit unchanged version entries',
          );
          await hot.check(false);
          assert.deepEqual(
            { ...versions },
            oldVersions,
            'prepare must not publish the new map',
          );
          await hot.apply();
          assert.equal(Object.hasOwn(versions, 'extra'), step === 1);
          assert.notEqual(versions.page, oldPage);
          if (step === 3)
            assert.deepEqual(
              { ...versions },
              oldVersions,
              'an unrelated edit must not change versions',
            );
        }
        assert.equal(assets['unchanged.js'], unchangedSource);
        oldVersions = { ...versions };
        oldHash = stats.hash;
        assert.equal((await state.api.load()).default, step ? 2 : 1);
      }
    } finally {
      delete globalThis[stateKey];
    }
  },
}));
