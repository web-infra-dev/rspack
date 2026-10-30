import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import vm from 'node:vm';
import { rspack } from '@rspack/core';

async function wait(promise) {
  let timer;
  try {
    return await Promise.race([
      promise,
      new Promise((_, reject) => {
        timer = setTimeout(() => reject(new Error('HMR did not settle')), 3000);
      }),
    ]);
  } finally {
    clearTimeout(timer);
  }
}

const scenarios = [
  ...[
    'late',
    'hashed-late',
    'during-prepare',
    'manual',
    'load-error',
    'two-updates',
    'removed',
    'moved',
    'manifest-error',
  ].map((name) => ({ name })),
  ...['before-check', 'check', 'prepare', 'downloading'].flatMap((start) =>
    [1, 2].flatMap((version) =>
      ['chunk-first', 'update-first'].map((order) => ({
        name: `${start}-v${version}-${order}`,
        start,
        version,
        order,
      })),
    ),
  ),
  ...['update-error', 'abort', 'accept-error', 'manual-apply'].map((name) => ({
    name,
    start: 'downloading',
    version: 2,
    order: 'chunk-first',
  })),
  { name: 'async-prepare', start: 'prepare', version: 1, order: 'chunk-first' },
  {
    name: 'manual-ready-import',
    start: 'ready',
    version: 1,
    order: 'update-first',
  },
  {
    name: 'ready-waits-import',
    start: 'downloading',
    version: 1,
    order: 'update-first',
  },
  ...['prepare-throw', 'prepare-reject'].map((name) => ({
    name,
    start: 'prepare',
    version: 2,
    order: 'chunk-first',
  })),
  ...['ready-throw', 'ready-reject', 'ready-starts-import'].map((name) => ({
    name,
    start: 'downloading',
    version: 2,
    order: 'update-first',
  })),
];

export default ['web', 'async-node', 'module'].flatMap((target) =>
  ['webpack', 'rspack'].flatMap((runtimeMode) =>
    scenarios.map(({ name: scenario, start, version, order }) => ({
      name: `${target}-${runtimeMode}-${scenario}`,
      description: 'applies HMR after an in-flight chunk installs',
      compiler(_context, compiler) {
        compiler.outputFileSystem = fs;
      },
      options(context) {
        const esm = target === 'module';
        return {
          context: context.getDist('input'),
          entry: './index.js',
          mode: 'development',
          devtool: false,
          target: esm ? 'node' : target,
          experiments: { runtimeMode },
          output: {
            path: context.getDist('output'),
            module: esm,
            chunkLoading: esm ? 'import' : undefined,
            importFunctionName: esm ? 'globalThis.hmrChunkImport' : undefined,
            filename: esm ? '[name].mjs' : '[name].js',
            chunkFilename: `[name]${scenario === 'hashed-late' ? '.[contenthash]' : ''}.${esm ? 'mjs' : 'js'}`,
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
            splitChunks: false,
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
        const write = (name, code) =>
          fs.writeFileSync(path.join(input, name), code);
        write(
          'index.js',
          `
          var app = globalThis.inFlightHmr = { hot: module.hot, hash: () => __webpack_hash__, executions: [] };
          app.load = () => import(/* webpackChunkName: "page" */ './page');
          app.other = () => import(/* webpackChunkName: "q" */ './q');
          module.hot.accept('./page');
          require('./trigger');
          ${scenario === 'abort' ? "module.hot.decline('./trigger');" : `module.hot.accept('./trigger', () => { ${scenario === 'accept-error' ? 'throw new Error("accept failed");' : "require('./trigger');"} });`}
        `,
        );
        write(
          'page.js',
          scenario === 'moved'
            ? 'module.exports = "page1-" + require("./m");'
            : `${scenario === 'removed' ? 'require("./removed");' : ''} module.exports = 1;`,
        );
        write('q.js', 'module.exports = "q1";');
        write('m.js', 'module.exports = "m1";');
        write('removed.js', 'globalThis.removedRan = true;');
        write('trigger.js', 'module.exports = 0;');
        const pageSource = (version) => `
          globalThis.inFlightHmr.executions.push({ version: ${version}, hash: __webpack_hash__, status: module.hot.status() });
          module.exports = ${version};
        `;
        if (start) write('page.js', pageSource(1));

        const native = target !== 'web';
        const sandbox = native
          ? globalThis
          : vm.createContext({ console, Promise, setTimeout, clearTimeout });
        const originalReadFile = fs.readFile;
        const isPage = (name) =>
          /^page(?:\.[a-f0-9]+)?\.(?:js|mjs)$/.test(name);
        const holdChunk = (name) =>
          isPage(name) ||
          (scenario === 'ready-starts-import' && /^q\.(?:js|mjs)$/.test(name));
        let initialPage;
        const held = [];
        const requested = [];
        const delivered = [];
        const requestWaiters = [];
        let holding = false;
        let resolveHeld;
        const heldResponse = new Promise((resolve) => {
          resolveHeld = resolve;
        });
        const hold = (deliver, fail, name = 'page') => {
          held.push({ deliver, fail, name });
          requested.push(name);
          resolveHeld();
          requestWaiters.splice(0).forEach((resolve) => resolve());
        };
        const response = (pattern) =>
          wait(
            (async () => {
              while (true) {
                const result = held.find((item) => pattern.test(item.name));
                if (result) return result;
                await new Promise((resolve) => requestWaiters.push(resolve));
              }
            })(),
          );
        const deliver = async (item) => {
          held.splice(held.indexOf(item), 1);
          delivered.push(item.name);
          await item.deliver();
        };
        const release = () => {
          holding = false;
          for (const response of held.splice(0)) {
            if (scenario === 'load-error') response.fail();
            else response.deliver();
          }
        };
        const manifestError = scenario === 'manifest-error';
        const networkError = () => new TypeError('Failed to fetch');
        if (target === 'web') {
          sandbox.fetch = async (url) => {
            if (manifestError) throw networkError();
            if (start === 'check' && holding)
              await new Promise((resolve, reject) =>
                hold(resolve, reject, path.basename(url)),
              );
            const filename = path.join(output, path.basename(url));
            return {
              status: fs.existsSync(filename) ? 200 : 404,
              ok: fs.existsSync(filename),
              json: async () => JSON.parse(fs.readFileSync(filename, 'utf8')),
            };
          };
          const scripts = [];
          const head = {
            appendChild(script) {
              scripts.push(script);
              script.parentNode = head;
              // Capture the response before the next build overwrites its file.
              const name = path.basename(script.src);
              const source = fs.readFileSync(path.join(output, name), 'utf8');
              const deliver = () => {
                const content =
                  start && name === 'page.js'
                    ? version === 1
                      ? initialPage
                      : fs.readFileSync(path.join(output, name), 'utf8')
                    : source;
                vm.runInContext(content, sandbox);
                script.onload?.({ type: 'load', target: script });
              };
              if (
                holding &&
                (holdChunk(name) || (start && name.endsWith('.hot-update.js')))
              ) {
                hold(
                  deliver,
                  () => script.onerror({ type: 'error', target: script }),
                  name,
                );
              } else queueMicrotask(deliver);
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
        } else if (target === 'async-node') {
          fs.readFile = function (filename, ...args) {
            const callback = args.pop();
            if (
              manifestError &&
              filename.startsWith(output) &&
              filename.endsWith('.hot-update.json')
            ) {
              queueMicrotask(() => callback(networkError()));
              return;
            }
            return originalReadFile.call(
              this,
              filename,
              ...args,
              (error, content) => {
                const name = path.basename(filename);
                if (
                  holding &&
                  args[0] === 'utf-8' &&
                  filename.startsWith(output) &&
                  (holdChunk(name) ||
                    (start && name.endsWith('.hot-update.js')) ||
                    (start === 'check' && name.endsWith('.hot-update.json')))
                ) {
                  hold(
                    () =>
                      callback(
                        error,
                        start && name === 'page.js'
                          ? version === 1
                            ? initialPage
                            : fs.readFileSync(filename, 'utf8')
                          : content,
                      ),
                    () => callback(networkError()),
                    name,
                  );
                } else callback(error, content);
              },
            );
          };
        } else {
          sandbox.hmrChunkImport = (url) => {
            url = new URL(url, pathToFileURL(`${output}/`)).href;
            if (manifestError && url.includes('hot-update.json'))
              return Promise.reject(networkError());
            const filename = fileURLToPath(url);
            const name = path.basename(filename);
            if (
              holding &&
              (holdChunk(name) ||
                (start && name.endsWith('.hot-update.mjs')) ||
                (start === 'check' && name.endsWith('.hot-update.json.mjs')))
            ) {
              const copy = path.join(output, `held-${name}`);
              fs.copyFileSync(filename, copy);
              return new Promise((resolve, reject) => {
                hold(
                  () => {
                    if (start && name === 'page.mjs')
                      fs.writeFileSync(
                        copy,
                        version === 1
                          ? initialPage
                          : fs.readFileSync(filename, 'utf8'),
                      );
                    return import(pathToFileURL(copy).href).then(
                      resolve,
                      reject,
                    );
                  },
                  () => reject(networkError()),
                  name,
                );
              });
            }
            return import(url);
          };
        }
        const build = async (changes = {}) => {
          Object.entries(changes).forEach(([name, code]) => write(name, code));
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
          return stats.hash;
        };
        try {
          await build();
          initialPage = fs.readFileSync(
            path.join(output, fs.readdirSync(output).find(isPage)),
            'utf8',
          );
          if (target === 'module')
            await import(pathToFileURL(path.join(output, 'main.mjs')).href);
          else if (native)
            createRequire(import.meta.url)(path.join(output, 'main.js'));
          else
            vm.runInContext(
              fs.readFileSync(path.join(output, 'main.js'), 'utf8'),
              sandbox,
            );
          const app = sandbox.inFlightHmr;
          const statuses = [];
          app.hot.addStatusHandler((status) => statuses.push(status));

          if (start) {
            holding = true;
            let imports;
            let pendingImports;
            let otherImport;
            const load = () => {
              pendingImports = [app.load(), app.load()];
              imports = Promise.all(pendingImports);
              imports.catch(() => {});
            };
            if (start === 'before-check') {
              load();
              await response(/^page\.(?:js|mjs)$/);
            }
            const nextHash = await build({
              'page.js': pageSource(2),
              'trigger.js': 'module.exports = 1;',
            });
            let finishPrepare;
            const prepareGate = new Promise((resolve) => {
              finishPrepare = resolve;
            });
            if (start === 'prepare')
              app.hot.addStatusHandler((status) => {
                if (status === 'prepare' && !imports) load();
                if (status === 'prepare' && scenario === 'async-prepare')
                  return prepareGate;
                if (status === 'prepare' && scenario === 'prepare-throw')
                  throw new Error('prepare failed');
                if (status === 'prepare' && scenario === 'prepare-reject')
                  return Promise.reject(new Error('prepare failed'));
              });
            if (start === 'ready')
              app.hot.addStatusHandler((status) => {
                if (status === 'ready' && !imports) {
                  load();
                  return imports;
                }
              });
            app.hot.addStatusHandler((status) => {
              if (status === 'ready' && scenario === 'ready-waits-import')
                return imports;
              if (
                status === 'ready' &&
                scenario === 'ready-starts-import' &&
                !otherImport
              ) {
                otherImport = app.other();
                otherImport.catch(() => {});
              }
              if (status === 'ready' && scenario === 'ready-throw')
                throw new Error('ready failed');
              if (status === 'ready' && scenario === 'ready-reject')
                return Promise.reject(new Error('ready failed'));
            });
            const update = app.hot.check(
              scenario !== 'manual-apply' &&
                scenario !== 'ready-waits-import' &&
                start !== 'ready',
            );
            update.catch(() => {});
            let page;
            if (scenario === 'prepare-throw' || scenario === 'prepare-reject') {
              page = await response(/^page\.(?:js|mjs)$/);
              await assert.rejects(wait(update), /prepare failed/);
              assert.equal(app.hot.status(), 'fail');
              await deliver(page);
              const results = await wait(Promise.allSettled(pendingImports));
              assert.ok(
                results.every((result) => result.status === 'rejected'),
              );
              assert.equal(
                app.hot.status(),
                'fail',
                'a late chunk must not revive failed HMR',
              );
              assert.equal(app.executions.length, 0);
              return;
            }
            if (start === 'check') {
              const manifest = await response(/\.hot-update\.json(?:\.mjs)?$/);
              load();
              page = await response(/^page\.(?:js|mjs)$/);
              if (order === 'chunk-first') {
                await deliver(page);
                await new Promise((resolve) => setImmediate(resolve));
                assert.equal(app.hot.status(), 'check');
                assert.equal(app.executions.length, 0);
              }
              await deliver(manifest);
            }
            if (scenario === 'async-prepare') {
              page = await response(/^page\.(?:js|mjs)$/);
              await deliver(page);
              await new Promise((resolve) => setImmediate(resolve));
              assert.equal(
                app.hot.status(),
                'prepare',
                'an async status handler must finish before ready',
              );
              finishPrepare();
            }
            const mainUpdate = await response(
              /^main\..*\.hot-update\.(?:js|mjs)$/,
            );
            if (start === 'downloading') load();
            if (start === 'ready') await deliver(mainUpdate);
            if (scenario.startsWith('ready-')) await deliver(mainUpdate);
            page ||= await response(/^page\.(?:js|mjs)$/);
            let pageUpdate;
            if (start === 'downloading' || start === 'ready') {
              pageUpdate = await response(/^page\..*\.hot-update\.(?:js|mjs)$/);
              if (order === 'update-first') await deliver(pageUpdate);
              await deliver(page);
              if (order === 'chunk-first') await deliver(pageUpdate);
            } else {
              if (held.includes(page))
                assert.equal(
                  requested.some((name) =>
                    /^page\..*\.hot-update\./.test(name),
                  ),
                  false,
                  'HMR must wait for a chunk that was already loading',
                );
              if (order === 'update-first') await deliver(mainUpdate);
              if (held.includes(page)) await deliver(page);
              pageUpdate = await response(/^page\..*\.hot-update\.(?:js|mjs)$/);
              await deliver(pageUpdate);
            }
            if (scenario === 'ready-starts-import') {
              const other = await response(/^q\.(?:js|mjs)$/);
              assert.equal(app.hot.status(), 'prepare');
              assert.equal(app.executions.length, 0);
              await deliver(other);
              assert.equal((await wait(otherImport)).default, 'q1');
            }
            // Drain promise callbacks while main's update still prevents apply.
            await new Promise((resolve) => setImmediate(resolve));
            if (held.includes(mainUpdate)) {
              assert.deepEqual(
                Array.from(app.executions),
                [],
                'the module must not execute before apply',
              );
              if (scenario === 'update-error') {
                held.splice(held.indexOf(mainUpdate), 1);
                mainUpdate.fail();
              } else await deliver(mainUpdate);
            }
            const failure = {
              'update-error': /Loading hot update chunk|Failed to fetch/,
              abort: /declined dependency/,
              'accept-error': /accept failed/,
              'ready-throw': /ready failed/,
              'ready-reject': /ready failed/,
            }[scenario];
            if (failure) {
              await assert.rejects(wait(update), failure);
              const results = await wait(Promise.allSettled(pendingImports));
              assert.ok(
                results.every((result) => result.status === 'rejected'),
              );
              assert.equal(
                app.hot.status(),
                scenario === 'abort' ? 'abort' : 'fail',
              );
              assert.equal(app.executions.length, 0);
              return;
            }
            await wait(update);
            if (start === 'ready' || scenario === 'ready-waits-import') {
              assert.deepEqual(
                (await wait(imports)).map((value) => value.default),
                [1, 1],
              );
              assert.equal(app.hot.status(), 'ready');
              await wait(app.hot.apply());
              assert.equal((await app.load()).default, 2);
              assert.deepEqual(
                Array.from(app.executions, (execution) => execution.version),
                [1, 2],
              );
              assert.equal(app.hash(), nextHash);
              assert.equal(app.hot.status(), 'idle');
              return;
            }
            if (scenario === 'manual-apply') {
              assert.equal(app.hot.status(), 'ready');
              assert.equal(app.executions.length, 0);
              await wait(app.hot.apply());
            }
            const values = await wait(imports);
            assert.deepEqual(
              values.map((value) => value.default),
              [2, 2],
            );
            assert.equal(app.executions.length, 1);
            assert.equal(app.executions[0].version, 2);
            assert.equal(app.executions[0].hash, nextHash);
            assert.equal(app.hash(), nextHash);
            assert.equal(app.hot.status(), 'idle');
            assert.equal((await app.load()).default, 2);
            assert.equal(
              requested.filter((name) => /^page\.(?:js|mjs)$/.test(name))
                .length,
              1,
            );
            const firstUpdate =
              start === 'downloading' ? pageUpdate : mainUpdate;
            assert.equal(
              delivered.indexOf(page.name) <
                delivered.indexOf(firstUpdate.name),
              order === 'chunk-first',
            );
            return;
          }

          if (manifestError) {
            await build({ 'page.js': 'module.exports = 2;' });
            const results = await wait(
              Promise.allSettled([app.load(), app.load()]),
            );
            assert.ok(results.every((result) => result.status === 'rejected'));
            assert.equal(app.hot.status(), 'fail');
            assert.deepEqual(statuses, ['check', 'fail']);
            return;
          }

          holding = true;
          const pending = app.load().then(
            (value) => ({ value }),
            (error) => ({ error }),
          );
          await wait(heldResponse);
          await build(
            scenario === 'moved'
              ? {
                  'page.js': 'module.exports = "page2";',
                  'q.js': 'module.exports = "q2-" + require("./m");',
                  'm.js': 'module.exports = "m2";',
                }
              : { 'page.js': 'module.exports = 2;' },
          );
          app.hot.addStatusHandler((status) => {
            if (status !== 'prepare' || !holding) return;
            if (
              ['during-prepare', 'removed', 'moved', 'two-updates'].includes(
                scenario,
              )
            )
              release();
            else setTimeout(release, 0);
          });
          await wait(app.hot.check(scenario !== 'manual'));
          if (scenario === 'manual') {
            assert.equal(app.hot.status(), 'ready');
            assert.equal((await wait(pending)).value.default, 1);
            await wait(app.hot.apply());
          } else {
            const result = await wait(pending);
            if (scenario === 'load-error') assert.ok(result.error);
            else {
              assert.equal(result.error, undefined);
              assert.equal(
                result.value.default,
                scenario === 'moved' ? 'page2' : 2,
              );
            }
          }
          // The existing async-node loader retains its rejected chunk promise.
          if (target !== 'async-node' || scenario !== 'load-error') {
            assert.equal(
              (await wait(app.load())).default,
              scenario === 'moved' ? 'page2' : 2,
            );
          }
          assert.equal(sandbox.removedRan, undefined);
          if (scenario === 'moved')
            assert.equal((await wait(app.other())).default, 'q2-m2');
          if (scenario === 'two-updates') {
            await build({ 'page.js': 'module.exports = 3;' });
            await wait(app.hot.check(true));
            assert.equal((await wait(app.load())).default, 3);
          }
          assert.equal(app.hot.status(), 'idle');
          assert.equal(
            statuses.filter((status) => status === 'check').length,
            scenario === 'two-updates' ? 2 : 1,
          );
        } finally {
          fs.readFile = originalReadFile;
          for (const response of held.splice(0)) response.fail();
          if (native) {
            delete globalThis.inFlightHmr;
            delete globalThis.hmrChunkImport;
            delete globalThis.removedRan;
          }
        }
      },
    })),
  ),
);
