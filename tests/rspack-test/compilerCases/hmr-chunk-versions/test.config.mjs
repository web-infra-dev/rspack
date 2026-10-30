import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import vm from 'node:vm';
import { rspack } from '@rspack/core';

export default ['web', 'webworker', 'node', 'async-node', 'module'].flatMap(
  (target) =>
    ['webpack', 'rspack'].flatMap((runtimeMode) =>
      [
        'target',
        'sibling',
        'initial-sibling',
        'multiple-updates',
        'concurrent-imports',
        'full-rebuild',
        'manual-apply',
        'missing-update',
        'aborted-update',
      ].map((scenario) => ({
        name: `${target}-${runtimeMode}-${scenario}`,
        description:
          'waits for compatible chunk versions before executing a dynamic import',
        compiler(_context, compiler) {
          compiler.outputFileSystem = fs;
        },
        options(context) {
          const esm = target === 'module';
          return {
            context: context.getDist('input'),
            entry:
              scenario === 'initial-sibling'
                ? { main: './index.js', other: './other.js' }
                : './index.js',
            mode: 'development',
            incremental: scenario !== 'full-rebuild',
            devtool: false,
            target: esm ? 'node' : target,
            experiments: { runtimeMode },
            output: {
              path: context.getDist('output'),
              module: esm,
              chunkLoading: esm ? 'import' : undefined,
              filename: esm ? '[name].mjs' : '[name].js',
              chunkFilename: esm ? '[name].mjs' : '[name].js',
              publicPath: esm
                ? pathToFileURL(`${context.getDist('output')}/`).href
                : '/',
              globalObject: 'globalThis',
            },
            optimization: {
              moduleIds: 'named',
              chunkIds: 'named',
              usedExports: false,
              concatenateModules: false,
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
          const write = (name, source) =>
            fs.writeFileSync(path.join(input, name), source);
          write(
            'index.js',
            `
					var app = globalThis.chunkVersionTest = { hot: module.hot, hash: () => __webpack_hash__ };
					app.load = () => import(/* webpackChunkName: "page" */ './page');
					require('./trigger');
					require('./block');
					module.hot.decline('./block');
					module.hot.accept('./trigger', () => {
						require('./trigger');
						app.pending = ${scenario === 'concurrent-imports' ? 'Promise.all([app.load(), app.load()]).then(modules => modules[0])' : 'app.load()'}.then(m => ({ value: m.default }), e => ({ error: e.message }));
					});
				`,
          );
          write('trigger.js', 'module.exports = 0;');
          write('block.js', 'module.exports = 0;');
          write('other.js', "require('./dep');");
          const sibling = scenario.includes('sibling');
          const valueSource = (value) =>
            `module.exports = { value: ${value}, hash: __webpack_hash__ };`;
          write(
            'page.js',
            sibling ? "module.exports = require('./dep');" : valueSource(1),
          );
          write('dep.js', valueSource(1));
          const assets = {};
          const native =
            target === 'node' || target === 'async-node' || target === 'module';
          const sandbox = native
            ? globalThis
            : vm.createContext({ console, Promise, setTimeout, clearTimeout });
          if (!native) {
            sandbox.importScripts = (url) =>
              vm.runInContext(assets[path.basename(url)], sandbox);
            sandbox.fetch = async (url) => {
              const content = assets[path.basename(url)];
              return {
                status: content === undefined ? 404 : 200,
                ok: content !== undefined,
                json: async () => JSON.parse(content),
              };
            };
            const scripts = [];
            const head = {
              appendChild(script) {
                scripts.push(script);
                script.parentNode = head;
                vm.runInContext(assets[path.basename(script.src)], sandbox);
                queueMicrotask(() =>
                  script.onload?.({ type: 'load', target: script }),
                );
              },
              removeChild(script) {
                scripts.splice(scripts.indexOf(script), 1);
              },
            };
            sandbox.document = {
              head,
              createElement: () => ({
                setAttribute(key, value) {
                  this[key] = value;
                },
                getAttribute(key) {
                  return this[key];
                },
              }),
              getElementsByTagName: () => scripts,
            };
          }
          const build = async (changes) => {
            for (const [name, source] of Object.entries(changes))
              write(name, source);
            const stats = await new Promise((resolve, reject) =>
              compiler.run(
                (error, stats) => (error ? reject(error) : resolve(stats)),
                {
                  modifiedFiles: new Set(
                    Object.keys(changes).map((name) => path.join(input, name)),
                  ),
                },
              ),
            );
            assert.equal(stats.hasErrors(), false, stats.toString());
            for (const asset of stats.compilation.getAssets())
              assets[asset.name] = asset.source.source().toString();
            return stats.hash;
          };
          try {
            const initialHash = await build({});
            if (target === 'module')
              await import(pathToFileURL(path.join(output, 'main.mjs')).href);
            else if (native)
              createRequire(import.meta.url)(path.join(output, 'main.js'));
            else vm.runInContext(assets['main.js'], sandbox);
            const app = sandbox.chunkVersionTest;
            const firstHash = await build({
              'trigger.js': 'module.exports = 1;',
            });
            if (scenario === 'multiple-updates')
              await build({ 'trigger.js': 'module.exports = 2;' });
            const changes = {
              [sibling ? 'dep.js' : 'page.js']: valueSource(2),
            };
            if (scenario === 'aborted-update')
              changes['block.js'] = 'module.exports = 1;';
            const latestHash = await build(changes);
            if (scenario === 'missing-update') {
              for (const name of Object.keys(assets)) {
                if (
                  name.includes(firstHash) &&
                  name.includes('hot-update.json')
                ) {
                  delete assets[name];
                  fs.unlinkSync(path.join(output, name));
                }
              }
            }
            let manualImport;
            if (scenario === 'manual-apply') {
              await app.hot.check(false);
              const ready = new Promise((resolve) => {
                const handler = (status) => {
                  if (status === 'ready') {
                    app.hot.removeStatusHandler(handler);
                    resolve();
                  }
                };
                app.hot.addStatusHandler(handler);
              });
              manualImport = app.load();
              await ready;
              assert.equal(app.hash(), initialHash);
              await app.hot.apply();
            } else await app.hot.check(true);
            const result = await app.pending;
            if (scenario === 'missing-update') {
              // Native import() propagates the missing manifest's module error.
              assert.match(
                result.error,
                target === 'module'
                  ? /Cannot find module .*hot-update\.json\.mjs/
                  : /no update leads to it/,
              );
            } else if (scenario === 'aborted-update') {
              assert.match(result.error, /declined dependency/);
              assert.equal(app.hot.status(), 'abort');
            } else {
              assert.equal(result.value.value, 2);
              assert.equal(
                result.value.hash,
                latestHash,
                'the module must execute after the runtime catches up',
              );
              assert.equal(app.hash(), latestHash);
              if (manualImport)
                assert.equal((await manualImport).default.hash, latestHash);
              assert.equal((await app.load()).default.hash, latestHash);
              assert.equal(app.hot.status(), 'idle');
            }
          } finally {
            if (native) delete globalThis.chunkVersionTest;
          }
        },
      })),
    ),
);
