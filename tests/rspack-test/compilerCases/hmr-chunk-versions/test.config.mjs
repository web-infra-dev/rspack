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
        'unchanged-chunk',
        'unchanged-hashed-chunk',
        'manual-apply',
        'missing-update',
        'aborted-update',
        ...(target === 'web' ? ['runtime-priority', 'catch-up-check'] : []),
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
              chunkFilename: `[name]${scenario === 'unchanged-hashed-chunk' ? '.[contenthash]' : ''}.${esm ? 'mjs' : 'js'}`,
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
					${scenario === 'runtime-priority' ? 'app.load = () => __webpack_require__.e("page", "high").then(() => ({ default: __webpack_require__("./page.js") }));' : ''}
					require('./trigger');
					require('./block');
					module.hot.decline('./block');
					module.hot.accept('./trigger', () => {
						require('./trigger');
						app.imports = ${['concurrent-imports', 'missing-update', 'aborted-update'].includes(scenario) ? '[app.load(), app.load()]' : '[app.load()]'};
						app.pending = Promise.all(app.imports).then(modules => ({ value: modules[0].default }), e => ({ error: e.message }));
					});
				`,
          );
          write('trigger.js', 'module.exports = 0;');
          write('block.js', 'module.exports = 0;');
          write('other.js', "require('./dep');");
          write('second.js', 'module.exports = 3;');
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
              vm.runInContext(
                assets[path.basename(url.split('?')[0])],
                sandbox,
              );
            sandbox.fetch = async (url) => {
              if (
                scenario === 'catch-up-check' &&
                sandbox.chunkVersionTest.hash() !== initialHash
              ) {
                concurrentCheck = Promise.resolve().then(() =>
                  sandbox.chunkVersionTest.hot.check(true),
                );
                concurrentCheck.catch(() => {});
              }
              const content = assets[path.basename(url.split('?')[0])];
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
                if (path.basename(script.src.split('?')[0]) === 'page.js')
                  sandbox.pagePriority =
                    script.fetchpriority || script.fetchPriority;
                vm.runInContext(
                  assets[path.basename(script.src.split('?')[0])],
                  sandbox,
                );
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
          let lastStats;
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
            lastStats = stats;
            for (const asset of stats.compilation.getAssets())
              assets[asset.name] = asset.source.source().toString();
            return stats.hash;
          };
          let initialHash;
          let concurrentCheck;
          try {
            initialHash = await build({});
            const initialPage = [...lastStats.compilation.chunks].find(
              (chunk) => chunk.name === 'page',
            );
            const pageHash = initialPage.hash;
            const pageFile = [...initialPage.files][0];
            const pageSource = assets[pageFile];
            if (target === 'module')
              await import(pathToFileURL(path.join(output, 'main.mjs')).href);
            else if (native)
              createRequire(import.meta.url)(path.join(output, 'main.js'));
            else vm.runInContext(assets['main.js'], sandbox);
            const app = sandbox.chunkVersionTest;
            if (scenario === 'runtime-priority') {
              await build({
                'trigger.js':
                  'module.exports = () => import(/* webpackFetchPriority: "high", webpackChunkName: "second" */ "./second");',
              });
              await app.hot.check(true);
              await app.pending;
              assert.equal(
                sandbox.pagePriority,
                'high',
                'an existing importer must call the updated ensureChunk',
              );
              return;
            }
            const firstHash = await build({
              'trigger.js': 'module.exports = 1;',
            });
            if (scenario.startsWith('unchanged-')) {
              const page = [...lastStats.compilation.chunks].find(
                (chunk) => chunk.name === 'page',
              );
              assert.equal(
                page.hash,
                pageHash,
                'HMR metadata must not change the chunk hash',
              );
              assert.deepEqual([...page.files], [pageFile]);
              assert.equal(
                assets[pageFile],
                pageSource,
                'an unrelated update must not change the chunk or its version',
              );
              assert.equal(
                fs.readFileSync(path.join(output, pageFile), 'utf8'),
                assets[pageFile],
                'the unchanged chunk must retain its emitted contents',
              );
              assert.equal(
                lastStats.hasWarnings(),
                false,
                lastStats.toString(),
              );
              await app.hot.check(true);
              const result = await app.pending;
              assert.equal(result.error, undefined);
              assert.equal(result.value.value, 1);
              assert.equal(result.value.hash, firstHash);
              assert.equal(app.hot.status(), 'idle');
              return;
            }
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
            const imports = await Promise.allSettled(app.imports);
            assert.ok(
              imports.every(
                (item) =>
                  item.status === (result.error ? 'rejected' : 'fulfilled'),
              ),
            );
            if (scenario === 'catch-up-check') {
              assert.ok(concurrentCheck);
              await assert.rejects(
                concurrentCheck,
                /check\(\) is only allowed in idle status/,
              );
            }
            if (scenario === 'missing-update') {
              // Native import() propagates the missing manifest's module error.
              const expectedError =
                target === 'module'
                  ? /Cannot find module .*hot-update\.json\.mjs/
                  : /no update leads to it/;
              assert.ok(
                imports.some((item) => expectedError.test(item.reason.message)),
              );
            } else if (scenario === 'aborted-update') {
              assert.ok(
                imports.some((item) =>
                  /declined dependency/.test(item.reason.message),
                ),
              );
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
