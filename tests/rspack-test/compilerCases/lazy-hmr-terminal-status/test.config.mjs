import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import vm from 'node:vm';
import { lazyCompilationMiddleware, rspack } from '@rspack/core';

async function wait(promise) {
  let timer;
  try {
    return await Promise.race([
      promise,
      new Promise((_, reject) => {
        timer = setTimeout(
          () => reject(new Error('Lazy import did not settle')),
          3000,
        );
      }),
    ]);
  } finally {
    clearTimeout(timer);
  }
}

export default ['node', 'async-node'].flatMap((target) =>
  ['webpack', 'rspack'].flatMap((runtimeMode) =>
    ['abort', 'fail', 'activate'].map((outcome) => {
      let middleware;
      return {
        name: `${target}-${runtimeMode}-${outcome}`,
        description:
          'settles lazy imports after HMR failure and preserves successful activation',
        options(context) {
          return {
            context: context.getDist('input'),
            entry: './index.js',
            mode: 'development',
            devtool: false,
            target,
            experiments: { runtimeMode },
            lazyCompilation: {
              entries: false,
              test: /(?:feature|unloaded)\.js$/,
              client: path.join(import.meta.dirname, 'client.js'),
            },
            output: {
              path: context.getDist('output'),
              filename: '[name].js',
              chunkFilename: '[name].js',
            },
            optimization: {
              moduleIds: 'named',
              chunkIds: 'named',
              splitChunks: false,
              concatenateModules: false,
            },
            plugins: [new rspack.HotModuleReplacementPlugin()],
          };
        },
        compiler(_context, compiler) {
          compiler.outputFileSystem = fs;
          middleware = lazyCompilationMiddleware(compiler);
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
          write('feature.js', 'module.exports = 42;');
          write('unloaded.js', 'module.exports = 43;');
          write('trigger.js', 'module.exports = 1;');
          write(
            'index.js',
            `
            require('./trigger');
            globalThis.lazyHmrTerminalStatus.hot = module.hot;
            globalThis.lazyHmrTerminalStatus.load = () => import(/* webpackChunkName: "feature" */ './feature');
            globalThis.lazyHmrTerminalStatus.loadFresh = () => import(/* webpackChunkName: "unloaded" */ './unloaded');
            ${
              outcome === 'abort'
                ? ''
                : `module.hot.accept('./trigger', () => {
              ${outcome === 'fail' ? 'throw new Error("Accept handler failed");' : ''}
            });`
            }
          `,
          );
          const build = async () => {
            const stats = await new Promise((resolve, reject) =>
              compiler.run(
                (error, stats) => (error ? reject(error) : resolve(stats)),
                {
                  modifiedFiles: new Set([path.join(input, 'trigger.js')]),
                },
              ),
            );
            assert.equal(stats.hasErrors(), false, stats.toString());
          };
          const state = (globalThis.lazyHmrTerminalStatus = {
            activated: [],
            disposed: 0,
          });
          try {
            await build();
            const filename = path.join(output, 'main.js');
            vm.runInThisContext(
              `(function(require, __dirname) { ${fs.readFileSync(filename, 'utf8')}\n})`,
            )(createRequire(filename), output);
            const activated = new Promise(
              (resolve) => (state.onActivate = resolve),
            );
            const pending = state.load();
            // Attach the rejection handler before applying the failing update.
            const result = pending.then(
              (value) => ({ value }),
              (error) => ({ error }),
            );
            await wait(activated);
            assert.equal(state.activated.length, 1);
            // A failed accept callback may run before the queued lazy activation
            // is compiled. Keep its proxy inactive in that scenario.
            if (outcome !== 'fail') {
              await middleware(
                {
                  url: '/_rspack/lazy/trigger',
                  method: 'POST',
                  body: [state.activated[0].data],
                },
                {
                  writeHead(status) {
                    assert.equal(status, 200);
                  },
                  write() {},
                  end() {},
                },
                () => assert.fail('lazy activation request was not handled'),
              );
            }
            write('trigger.js', 'module.exports = 2;');
            await build();

            if (outcome === 'activate') {
              await wait(state.hot.check(true));
              assert.equal((await wait(result)).value.default, 42);
              assert.equal(state.hot.status(), 'idle');
              assert.equal(state.disposed, 1);
            } else {
              await assert.rejects(
                wait(state.hot.check(true)),
                outcome === 'abort' ? /not accepted/ : /Accept handler failed/,
              );
              assert.equal(state.hot.status(), outcome);
              const { error } = await wait(result);
              assert(error, 'the pending lazy import must reject');
              assert.match(error.message, /cannot finish.*Reload the page/);
              await assert.rejects(wait(state.load()), /Reload the page/);
              // This proxy has not run before: it must also reject immediately
              // when instantiated after the runtime has already failed.
              await assert.rejects(wait(state.loadFresh()), /Reload the page/);
              assert.equal(state.activated.length, 2);
            }
          } finally {
            delete globalThis.lazyHmrTerminalStatus;
          }
        },
      };
    }),
  ),
);
