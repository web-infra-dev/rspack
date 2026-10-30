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

export default ['web', 'async-node', 'module'].flatMap((target) =>
  ['webpack', 'rspack'].flatMap((runtimeMode) =>
    [
      'late',
      'during-prepare',
      'manual',
      'load-error',
      'two-updates',
      'removed',
      'moved',
      'manifest-error',
    ].map((scenario) => ({
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
          var app = globalThis.inFlightHmr = { hot: module.hot };
          app.load = () => import(/* webpackChunkName: "page" */ './page');
          app.other = () => import(/* webpackChunkName: "q" */ './q');
          module.hot.accept('./page');
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

        const native = target !== 'web';
        const sandbox = native
          ? globalThis
          : vm.createContext({ console, Promise, setTimeout, clearTimeout });
        const originalReadFile = fs.readFile;
        const held = [];
        let holding = false;
        let resolveHeld;
        const heldResponse = new Promise((resolve) => {
          resolveHeld = resolve;
        });
        const hold = (deliver, fail) => {
          held.push({ deliver, fail });
          resolveHeld();
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
                vm.runInContext(source, sandbox);
                script.onload?.({ type: 'load', target: script });
              };
              if (holding && name === 'page.js') {
                hold(deliver, () =>
                  script.onerror({ type: 'error', target: script }),
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
                if (holding && filename === path.join(output, 'page.js')) {
                  hold(
                    () => callback(error, content),
                    () => callback(networkError()),
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
            if (holding && path.basename(fileURLToPath(url)) === 'page.mjs') {
              const copy = path.join(output, 'held-page.mjs');
              fs.copyFileSync(fileURLToPath(url), copy);
              return new Promise((resolve, reject) => {
                hold(
                  () => import(pathToFileURL(copy).href).then(resolve, reject),
                  () => reject(networkError()),
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
        };
        try {
          await build();
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
