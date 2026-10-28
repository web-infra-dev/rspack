import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { rspack } from '@rspack/core';

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description: 'installs complete ESM chunks and keeps receiving their updates',
  compiler(_context, compiler) {
    compiler.outputFileSystem = fs;
  },
  options(context) {
    return {
      context: context.getDist('input'),
      entry: './index.mjs',
      mode: 'development',
      target: 'node',
      devtool: false,
      output: {
        path: context.getDist('output'),
        module: true,
        library: { type: 'module' },
        chunkLoading: 'import',
        filename: 'main.mjs',
        chunkFilename: '[name].[contenthash].mjs',
        hotUpdateChunkFilename: '[id].[fullhash].hot-update.mjs',
        hotUpdateMainFilename: '[runtime].[fullhash].hot-update.json.mjs',
        publicPath: pathToFileURL(`${context.getDist('output')}/`).href,
      },
      optimization: {
        moduleIds: 'named',
        chunkIds: 'named',
        concatenateModules: false,
        splitChunks: {
          cacheGroups: {
            default: false,
            defaultVendors: false,
            shared: { test: /[\\/]dep\.js$/, name: 'shared', chunks: 'async', enforce: true },
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
      export const loadPage = () => import(/* webpackChunkName: 'page' */ './page.js');
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
          entry = await import(pathToFileURL(context.getDist('output/main.mjs')).href);
          await entry.loadPage();
        } else {
          await entry.hot.check(true);
        }
        assert.equal(globalThis.completeHmrModuleValue, [1, 2, 9, 9, 4][step]);
        assert.equal(entry.hot.status(), 'idle');
      }
    } finally {
      delete globalThis.completeHmrModuleValue;
    }
  },
};
