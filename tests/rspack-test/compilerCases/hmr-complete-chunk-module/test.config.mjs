import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { rspack } from '@rspack/core';

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default ['module', 'node', 'async-node'].flatMap(target => ['webpack', 'rspack'].map(runtimeMode => ({
  description: `installs complete entry dependencies and receives later updates (${target}, ${runtimeMode})`,
  compiler(_context, compiler) {
    compiler.outputFileSystem = fs;
  },
  options(context) {
    return {
      context: context.getDist('input'),
      entry: './index.mjs',
      mode: 'development',
      target: target === 'module' ? 'node' : target,
      experiments: { runtimeMode },
      devtool: false,
      output: {
        path: context.getDist(`output-${target}-${runtimeMode}`),
        module: target === 'module',
        library: { type: target === 'module' ? 'module' : 'commonjs2' },
        chunkLoading: target === 'module' ? 'import' : undefined,
        filename: target === 'module' ? '[name].mjs' : '[name].js',
        chunkFilename: `[name].[contenthash].${target === 'module' ? 'mjs' : 'js'}`,
        hotUpdateChunkFilename: `[id].[fullhash].hot-update.${target === 'module' ? 'mjs' : 'js'}`,
        hotUpdateMainFilename: `[runtime].[fullhash].hot-update.json${target === 'module' ? '.mjs' : ''}`,
        publicPath: pathToFileURL(`${context.getDist(`output-${target}-${runtimeMode}`)}/`).href,
      },
      optimization: {
        runtimeChunk: 'single',
        moduleIds: 'named',
        chunkIds: 'named',
        concatenateModules: false,
        splitChunks: {
          cacheGroups: {
            default: false,
            defaultVendors: false,
            shared: { test: /[\\/]dep\.js$/, name: 'constructor', chunks: 'all', enforce: true },
          },
        },
      },
      plugins: [new rspack.HotModuleReplacementPlugin()],
    };
  },
  async build(context, compiler) {
    const input = context.getDist('input');
    fs.mkdirSync(input, { recursive: true });
    // Even in a CommonJS package, supplemental ESM assets keep the .mjs extension.
    fs.writeFileSync(context.getDist('package.json'), '{"type":"commonjs"}');
    const write = (name, source) => fs.writeFileSync(path.join(input, name), source);
    write('index.mjs', `
      export const hot = import.meta.webpackHot;
      import './page.js';
      export const loadOther = () => import(/* webpackChunkName: 'other' */ './other.js');
      import.meta.webpackHot.accept();
    `);
    write('dep.js', 'module.exports = 2; module.hot.accept();');
    write('other.js', 'module.exports = require("./dep");');
    let entry;
    try {
      const values = ['1', 'require("./dep")', '9', '9', 'require("./dep") + 1'];
      for (let step = 0; step < values.length; step++) {
        write('page.js', `globalThis.completeHmrModuleValue = ${values[step]}; module.hot.accept();`);
        if (step === 3) write('dep.js', 'module.exports = 3; module.hot.accept();');
        const stats = await new Promise((resolve, reject) => compiler.run(
          (error, stats) => error ? reject(error) : resolve(stats),
          { modifiedFiles: new Set(['page.js', 'dep.js'].map(name => path.join(input, name))) },
        ));
        assert.equal(stats.hasErrors(), false, stats.toString());
        if (!step) {
          const loaded = await import(pathToFileURL(context.getDist(`output-${target}-${runtimeMode}/main.${target === 'module' ? 'mjs' : 'js'}`)).href);
          entry = target === 'module' ? loaded : await loaded.default;
        } else {
          if (step === 1) {
            const assets = stats.compilation.getAssets();
            const manifest = assets.find(asset => asset.name.includes('.hot-update.json'));
            assert.match(manifest.source.source().toString(), /"load":\{[^}]*"main":\["constructor"\]/);
            const extension = target === 'module' ? 'mjs' : 'js';
            assert(assets.some(asset => asset.name.startsWith('constructor.') && asset.name.endsWith(`.full.${extension}`)));
            assert(!assets.some(asset => asset.name.startsWith('constructor.') && asset.name.endsWith(`.hot-update.${extension}`)));
          }
          await entry.hot.check(true);
        }
        assert.equal(globalThis.completeHmrModuleValue, [1, 2, 9, 9, 4][step]);
        assert.equal(entry.hot.status(), 'idle');
      }
    } finally {
      delete globalThis.completeHmrModuleValue;
    }
  },
})));
