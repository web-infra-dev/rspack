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

// Request timing, ordinary response version, and response order are independent.
const cases = [
  { name: 'before-check-v1', phase: 'before', version: 1 },
  { name: 'before-check-v2', phase: 'before', version: 2 },
  { name: 'prepare-v1-chunk-first', version: 1 },
  { name: 'prepare-v2-chunk-first', version: 2 },
  { name: 'prepare-v1-update-first', version: 1, updateFirst: true },
  { name: 'prepare-v2-update-first', version: 2, updateFirst: true },
  { name: 'during-check', phase: 'check', version: 2 },
  { name: 'concurrent-after-apply', phase: 'late', version: 1 },
  { name: 'manifest-error', phase: 'check', version: 2 },
  { name: 'update-error', version: 2 },
  { name: 'retry-error', version: 2 },
  { name: 'retry-stale', version: 2 },
  { name: 'manual-compatible', version: 1, manual: true },
  { name: 'manual-retry', version: 2, manual: true },
  { name: 'ready-compatible', version: 1, updateFirst: true },
  { name: 'ready-future', version: 2, updateFirst: true },
  {
    name: 'ready-future-manual',
    version: 2,
    updateFirst: true,
    manual: true,
  },
  { name: 'ready-catch', version: 2, updateFirst: true },
  { name: 'ready-async', version: 2, updateFirst: true },
];

export default ['web', 'async-node', 'module'].flatMap((target) =>
  ['webpack', 'rspack'].flatMap((runtimeMode) =>
    cases.filter(({ name }) =>
      // Native import() reuses the first namespace, so there is no retry request.
      target !== 'module' || (name !== 'retry-error' && name !== 'retry-stale'),
    ).map(({ name, phase = 'prepare', version, updateFirst, manual }) => ({
      name: `${target}-${runtimeMode}-${name}`,
      description:
        'validates before installation and retries without blocking apply',
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
            importFunctionName: esm ? 'globalThis.raceImport' : undefined,
            filename: esm ? '[name].mjs' : '[name].js',
            chunkFilename: esm ? '[name].mjs' : '[name].js',
            publicPath: esm
              ? pathToFileURL(`${context.getDist('output')}/`).href
              : '/',
            globalObject: 'globalThis',
            chunkLoadTimeout: name.startsWith('ready-') ? 500 : 120000,
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
        const output = context.getDist('output');
        const extension = target === 'module' ? 'mjs' : 'js';
        const pageFile = `page.${extension}`;
        fs.mkdirSync(input, { recursive: true });
        fs.writeFileSync(
          context.getDist('package.json'),
          '{"type":"commonjs"}',
        );
        fs.writeFileSync(
          path.join(input, 'index.js'),
          `
          globalThis.raceApp = {
            hot: module.hot, hash: () => __webpack_hash__, executions: [],
            load: () => import(/* webpackChunkName: "page" */ './page'),
            factory: () => __webpack_require__.m['./page.js']
          };
          require('./trigger');
          module.hot.accept(['./trigger', './page']);
        `,
        );
        const build = async (step) => {
          fs.writeFileSync(
            path.join(input, 'trigger.js'),
            `module.exports = ${step};`,
          );
          fs.writeFileSync(
            path.join(input, 'page.js'),
            `
            globalThis.raceApp.executions.push(${step}); module.exports = ${step};
          `,
          );
          const stats = await new Promise((resolve, reject) =>
            compiler.run(
              (error, stats) => (error ? reject(error) : resolve(stats)),
              {
                modifiedFiles: new Set(
                  ['page.js', 'trigger.js'].map((file) =>
                    path.join(input, file),
                  ),
                ),
              },
            ),
          );
          assert.equal(stats.hasErrors(), false, stats.toString());
          return Object.fromEntries(
            stats.compilation
              .getAssets()
              .map(({ name, source }) => [name, source.source().toString()]),
          );
        };
        const initial = await build(1);
        let assets = initial;
        const sandbox =
          target === 'web'
            ? vm.createContext({ console, setTimeout, clearTimeout })
            : globalThis;
        const readFile = fs.readFile;
        const pending = [];
        let notify,
          attempts = 0,
          sequence = 0;
        const failure = () => new Error('Download failed');
        // Each transport captures an ordinary response independently of HMR.
        const request = (file, deliver, reject) => {
          const first = file === pageFile && ++attempts === 1;
          let settled = false;
          const response = {
            file,
            deliver() {
              if (settled) return;
              settled = true;
              if (file === pageFile && !first && name === 'retry-error')
                return reject(failure());
              const source =
                file === pageFile &&
                ((first && version === 1) || (!first && name === 'retry-stale'))
                  ? initial[file]
                  : assets[file];
              if (source === undefined) return reject(failure());
              return deliver(source);
            },
            reject() {
              if (settled) return;
              settled = true;
              reject(failure());
            },
          };
          if (
            first ||
            (file.includes('.hot-update.') && !file.includes('.json'))
          ) {
            pending.push(response);
            notify?.();
          } else if (phase === 'check' && file.includes('.hot-update.json')) {
            pending.push(response);
            notify?.();
          } else queueMicrotask(() => response.deliver());
        };
        const take = (match) =>
          wait(
            (async () => {
              while (true) {
                const index = pending.findIndex(({ file }) => match(file));
                if (index >= 0) return pending.splice(index, 1)[0];
                await new Promise((resolve) => {
                  notify = resolve;
                });
              }
            })(),
          );
        if (target === 'web') {
          sandbox.fetch = (url) =>
            new Promise((resolve, reject) =>
              request(
                path.basename(url),
                (source) =>
                  resolve({
                    ok: true,
                    status: 200,
                    json: async () => JSON.parse(source),
                  }),
                reject,
              ),
            );
          const scripts = [];
          const head = {
            appendChild(script) {
              scripts.push(script);
              script.parentNode = head;
              request(
                path.basename(script.src),
                (source) => {
                  vm.runInContext(source, sandbox);
                  script.onload?.({ type: 'load', target: script });
                },
                () => script.onerror?.({ type: 'error', target: script }),
              );
            },
            removeChild(script) {
              scripts.splice(scripts.indexOf(script), 1);
            },
          };
          sandbox.document = {
            head,
            getElementsByTagName: () => scripts,
            createElement: () => ({
              setAttribute(key, value) {
                this[key] = value;
              },
              getAttribute(key) {
                return this[key];
              },
            }),
          };
        } else if (target === 'async-node') {
          fs.readFile = function (file, ...args) {
            if (!file.startsWith(output) || args[0] !== 'utf-8')
              return readFile.call(this, file, ...args);
            const callback = args.pop();
            request(
              path.basename(file),
              (source) => callback(null, source),
              callback,
            );
          };
        } else {
          const moduleCache = new Map();
          sandbox.raceImport = (url) => {
            const key = new URL(url, pathToFileURL(`${output}/`)).href;
            if (moduleCache.has(key)) return moduleCache.get(key);
            const promise = new Promise((resolve, reject) => {
              const file = path.basename(
                fileURLToPath(new URL(url, pathToFileURL(`${output}/`))),
              );
              request(
                file,
                (source) => {
                  // Cache namespaces by request URL, as native import() does.
                  const response = path.join(output, `${sequence++}-${file}`);
                  fs.writeFileSync(response, source);
                  import(pathToFileURL(response).href).then(resolve, reject);
                },
                reject,
              );
            });
            moduleCache.set(key, promise);
            return promise;
          };
        }
        let imports, update;
        const held = [];
        try {
          if (target === 'web') vm.runInContext(initial['main.js'], sandbox);
          else if (target === 'module')
            await import(pathToFileURL(path.join(output, 'main.mjs')).href);
          else createRequire(import.meta.url)(path.join(output, 'main.js'));
          const app = sandbox.raceApp;
          const load = () => {
            imports = Promise.all([app.load(), app.load()]);
            imports.catch(() => {});
          };
          if (phase === 'before' || phase === 'late') load();
          assets = await build(2);
          if (name.startsWith('ready-'))
            app.hot.addStatusHandler((status) => {
              if (status !== 'ready') return;
              if (name === 'ready-async')
                return new Promise((resolve) => setTimeout(resolve, 20));
              return name === 'ready-catch' ? imports.catch(() => {}) : imports;
            });
          update = app.hot.check(!manual);
          update.catch(() => {});
          if (phase === 'check') {
            const manifest = await take((file) =>
              file.includes('.hot-update.json'),
            );
            held.push(manifest);
            load();
            const page = await take((file) => file === pageFile);
            await page.deliver();
            await new Promise((resolve) => setImmediate(resolve));
            assert.equal(
              app.factory(),
              undefined,
              'a future factory must not be installed',
            );
            if (name === 'manifest-error') {
              manifest.reject();
              await assert.rejects(wait(update), /Download failed/);
              await assert.rejects(wait(imports), /Download failed/);
              assert.equal(app.executions.length, 0);
              return;
            }
            await manifest.deliver();
          }
          const mainUpdate = await take(
            (file) => file.startsWith('main.') && file.includes('.hot-update.'),
          );
          held.push(mainUpdate);
          if (phase === 'prepare') load();
          // A future response rejected during check is no longer installed when
          // the manifest is handled, so its hot-update is not downloaded.
          const pageUpdate =
            phase === 'check'
              ? undefined
              : await take(
                  (file) =>
                    file.startsWith('page.') && file.includes('.hot-update.'),
                );
          const page =
            phase === 'check'
              ? undefined
              : await take((file) => file === pageFile);
          held.push(...[page, pageUpdate].filter(Boolean));
          const deliverUpdates = async () => {
            await pageUpdate?.deliver();
            if (name === 'update-error') mainUpdate.reject();
            else await mainUpdate.deliver();
          };
          if (updateFirst || phase === 'late') await deliverUpdates();
          if (phase === 'late') {
            await wait(update);
            const later = app.load();
            later.catch(() => {});
            imports = Promise.all([imports, later]).then(([first, last]) => [
              ...first,
              last,
            ]);
            imports.catch(() => {});
          }
          if (page) {
            await page.deliver();
            await new Promise((resolve) => setImmediate(resolve));
            if (version === 2 && !updateFirst) {
              assert.equal(
                app.factory(),
                undefined,
                'a future factory must not be installed',
              );
              assert.equal(
                app.hot.status(),
                'prepare',
                'version rejection must still wait for hot-update',
              );
            }
          }
          if (!updateFirst && phase !== 'late') await deliverUpdates();
          if (name === 'update-error') {
            await assert.rejects(wait(update));
            await assert.rejects(wait(imports));
            assert.equal(app.executions.length, 0);
            return;
          }
          if (name.startsWith('ready-future')) {
            await assert.rejects(wait(imports), {
              name: 'ChunkLoadError',
              type: 'timeout',
              message: /timed out waiting for HMR/,
            });
            await assert.rejects(wait(update), /timed out waiting for HMR/);
            assert.equal(app.factory(), undefined);
            assert.equal(app.executions.length, 0);
            assert.equal(attempts, 1);
            return;
          }
          await wait(update);
          if (manual) {
            assert.equal(app.hot.status(), 'ready');
            if (version === 1)
              assert.deepEqual(
                (await wait(imports)).map((m) => m.default),
                [1, 1],
              );
            else assert.equal(app.executions.length, 0);
            await app.hot.apply();
          }
          if (name === 'ready-catch') {
            await assert.rejects(wait(imports), /timed out waiting for HMR/);
            assert.equal(app.executions.length, 0);
            assert.equal((await wait(app.load())).default, 2);
            assert.equal(app.hot.status(), 'idle');
          } else if (target === 'module' && phase === 'late') {
            // The stable URL still resolves to V1 in the native module cache.
            // Reject instead of installing stale factories or waiting forever.
            await assert.rejects(wait(imports), /no update leads|Download failed/);
            assert.equal(attempts, 1);
            assert.equal(app.executions.length, 0);
          } else if (name === 'retry-error' || name === 'retry-stale') {
            await assert.rejects(
              wait(imports),
              /Loading chunk|Download failed|no update leads/,
            );
            assert.equal(attempts, 2);
            assert.equal(app.executions.length, 0);
          } else {
            const modules = await wait(imports);
            assert.ok(
              modules.every(
                (m) => m.default === 2 || (version === 1 && m.default === 1),
              ),
            );
            assert.equal((await app.load()).default, 2);
            const executions = Array.from(app.executions);
            // A compatible V1 response may execute before apply, then normal
            // HMR must dispose its cache before V2 executes.
            assert.deepEqual(executions, executions[0] === 1 ? [1, 2] : [2]);
            assert.equal(
              attempts,
              target !== 'module' && (phase === 'late' || version === 2) ? 2 : 1,
            );
            assert.equal(app.hot.status(), 'idle');
          }
        } finally {
          fs.readFile = readFile;
          for (const response of [...pending, ...held]) response.reject();
          imports?.catch(() => {});
          update?.catch(() => {});
          if (target !== 'web') {
            delete sandbox.raceApp;
            delete sandbox.raceImport;
          }
        }
      },
    })),
  ),
);
