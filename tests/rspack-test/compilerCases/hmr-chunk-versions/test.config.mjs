import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import vm from 'node:vm';
import { rspack } from '@rspack/core';

// Async loaders use the controlled-response race suite. Cover synchronous
// require/importScripts here, including a chunk initial in another runtime.
export default ['node', 'webworker'].flatMap((target) =>
  ['webpack', 'rspack'].flatMap((runtimeMode) =>
    [false, true].map((hashed) => ({
      name: `${target}-${runtimeMode}-${hashed ? 'contenthash' : 'stable'}`,
      description:
        'catches up before executing a shared module from a newer build',
      compiler(_context, compiler) {
        compiler.outputFileSystem = fs;
      },
      options(context) {
        return {
          context: context.getDist('input'),
          entry: { main: './index.js', other: './other.js' },
          mode: 'development',
          devtool: false,
          target,
          experiments: { runtimeMode },
          output: {
            path: context.getDist('output'),
            filename: '[name].js',
            chunkFilename: hashed ? '[name].[contenthash].js' : '[name].js',
            publicPath: '/',
            globalObject: 'globalThis',
          },
          optimization: {
            moduleIds: 'named',
            chunkIds: 'named',
            usedExports: false,
            splitChunks: {
              cacheGroups: {
                default: false,
                defaultVendors: false,
                shared: {
                  test: /[\\/]dep\.js$/,
                  name: 'shared',
                  chunks: 'all',
                  enforce: true,
                },
              },
            },
          },
          plugins: [new rspack.HotModuleReplacementPlugin()],
        };
      },
      async build(context, compiler) {
        const input = context.getDist('input');
        const output = context.getDist('output');
        fs.mkdirSync(input, { recursive: true });
        fs.writeFileSync(
          context.getDist('package.json'),
          '{"type":"commonjs"}',
        );
        const write = (file, source) =>
          fs.writeFileSync(path.join(input, file), source);
        write(
          'index.js',
          `
          var app = globalThis.versionApp = {
            hot: module.hot, hash: () => __webpack_hash__,
            ensure: () => __webpack_require__.e,
            otherEnsure: require('./ensure'),
            load: () => import(/* webpackChunkName: "page" */ './page')
          };
          require('./trigger');
          module.hot.accept('./trigger', () => {
            app.pending = Promise.all([app.load(), app.load()]);
          });
        `,
        );
        write('ensure.js', 'module.exports = () => __webpack_require__.e;');
        write('trigger.js', 'module.exports = 1;');
        write('page.js', 'module.exports = require("./dep");');
        write('other.js', 'require("./dep");');
        write(
          'dep.js',
          'module.exports = { value: 1, hash: __webpack_hash__ };',
        );
        const assets = {};
        const build = async (changes) => {
          for (const [file, source] of Object.entries(changes))
            write(file, source);
          const stats = await new Promise((resolve, reject) =>
            compiler.run(
              (error, stats) => (error ? reject(error) : resolve(stats)),
              {
                modifiedFiles: new Set(
                  Object.keys(changes).map((file) => path.join(input, file)),
                ),
              },
            ),
          );
          assert.equal(stats.hasErrors(), false, stats.toString());
          for (const { name, source } of stats.compilation.getAssets())
            assets[name] = source.source().toString();
          return stats;
        };
        await build({});
        const sandbox =
          target === 'node'
            ? globalThis
            : vm.createContext({ console, setTimeout, clearTimeout });
        if (target === 'node')
          createRequire(import.meta.url)(path.join(output, 'main.js'));
        else {
          sandbox.importScripts = (url) =>
            vm.runInContext(assets[path.basename(url)], sandbox);
          sandbox.fetch = async (url) => ({
            ok: assets[path.basename(url)] !== undefined,
            status: 200,
            json: async () => JSON.parse(assets[path.basename(url)]),
          });
          vm.runInContext(assets['main.js'], sandbox);
        }
        try {
          const app = sandbox.versionApp;
          const unchanged = Object.fromEntries(
            Object.entries(assets).filter(([file]) =>
              /^(page|shared)\./.test(file),
            ),
          );
          await build({ 'trigger.js': 'module.exports = 2;' });
          for (const [file, source] of Object.entries(unchanged)) {
            assert.equal(
              assets[file],
              source,
              'unrelated updates must preserve chunk versions and contents',
            );
          }
          // The browser starts at V1, applies V2, and encounters V3 in its accept callback.
          const latest = await build({
            'dep.js': 'module.exports = { value: 2, hash: __webpack_hash__ };',
          });
          await app.hot.check(true);
          const modules = await app.pending;
          assert.ok(
            modules.every(
              (m) => m.default.value === 2 && m.default.hash === latest.hash,
            ),
          );
          assert.equal(app.hash(), latest.hash);
          assert.equal(app.hot.status(), 'idle');
          assert.equal(
            app.ensure()('page'),
            app.otherEnsure()('page'),
            'concurrent ensures from different modules must share the load',
          );
        } finally {
          if (target === 'node') delete sandbox.versionApp;
        }
      },
    })),
  ),
);
