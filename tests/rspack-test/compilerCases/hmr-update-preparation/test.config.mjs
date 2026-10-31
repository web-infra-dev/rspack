import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { rspack } from '@rspack/core';

async function wait(promise) {
  let timer;
  try {
    return await Promise.race([
      promise,
      new Promise((_, reject) => {
        timer = setTimeout(() => reject(new Error('HMR did not settle')), 2000);
      }),
    ]);
  } finally {
    clearTimeout(timer);
  }
}

export default ['webpack', 'rspack'].flatMap((runtimeMode) =>
  [
    'manifest-error',
    'update-error',
    'async-prepare',
    'prepare-throw',
    'prepare-reject',
    'ready-throw',
    'ready-reject',
    'ready-waits-import',
    'dynamic-update-error',
    'ordinary-load-error',
  ].map((scenario) => ({
    name: `${runtimeMode}-${scenario}`,
    description: 'keeps update preparation pending and propagates its failures',
    options(context) {
      return {
        context: context.getDist('input'),
        entry: './index.js',
        mode: 'development',
        devtool: false,
        cache: false,
        target: 'web',
        experiments: { runtimeMode },
        output: {
          path: context.getDist('output'),
          filename: '[name].js',
          chunkFilename: '[name].js',
          publicPath: '/',
        },
        optimization: { moduleIds: 'named', chunkIds: 'named' },
        plugins: [new rspack.HotModuleReplacementPlugin()],
      };
    },
    async build(context, compiler) {
      const input = context.getDist('input');
      fs.mkdirSync(input, { recursive: true });
      fs.writeFileSync(path.join(input, 'package.json'), '{"type":"commonjs"}');
      fs.writeFileSync(
        path.join(input, 'index.js'),
        `var moduleRequire = __webpack_require__;
        globalThis.app = {
          hot: module.hot,
          chunk: () => import(/* webpackChunkName: "page" */ './page'),
          load: () => moduleRequire.e('page'),
          value: () => require('./trigger')
        };
        require('./trigger');
        module.hot.accept('./trigger');`,
      );
      const assets = {};
      const build = async (version) => {
        for (const name of ['trigger', 'page']) {
          fs.writeFileSync(path.join(input, `${name}.js`), `module.exports = ${version};`);
        }
        const stats = await new Promise((resolve, reject) =>
          compiler.run((error, stats) => error ? reject(error) : resolve(stats), {
            modifiedFiles: new Set(['trigger.js', 'page.js'].map((name) => path.join(input, name))),
          }),
        );
        assert.equal(stats.hasErrors(), false, stats.toString());
        for (const asset of stats.compilation.getAssets()) {
          assets[asset.name] = asset.source.source().toString();
        }
      };
      await build(1);
      const sandbox = vm.createContext({ console, setTimeout, clearTimeout });
      sandbox.self = sandbox;
      sandbox.fetch = async (url) => {
        if (scenario === 'manifest-error') throw new Error('manifest failed');
        const source = assets[path.basename(url.split('?')[0])];
        return { status: source ? 200 : 404, ok: Boolean(source), json: async () => JSON.parse(source) };
      };
      const scripts = [];
      const pending = [];
      let notify;
      const head = {
        appendChild(script) {
          scripts.push(script);
          script.parentNode = head;
          const name = path.basename(script.src.split('?')[0]);
          const source = assets[name];
          pending.push({
            name,
            deliver() {
              vm.runInContext(source, sandbox, { filename: name });
              script.onload?.({ type: 'load', target: script });
            },
            fail() { script.onerror?.({ type: 'error', target: script }); },
          });
          notify?.();
        },
        removeChild(script) { scripts.splice(scripts.indexOf(script), 1); },
      };
      sandbox.document = {
        head,
        getElementsByTagName: () => scripts,
        createElement: () => ({
          setAttribute(key, value) { this[key] = value; },
          getAttribute(key) { return this[key]; },
        }),
      };
      const take = (pattern) => wait((async () => {
        while (true) {
          const index = pending.findIndex(({ name }) => pattern.test(name));
          if (index >= 0) return pending.splice(index, 1)[0];
          await new Promise((resolve) => { notify = resolve; });
        }
      })());
      vm.runInContext(assets['main.js'], sandbox);
      const { app } = sandbox;
      const statuses = [];
      app.hot.addStatusHandler((status) => statuses.push(status));
      await build(2);
      let releasePrepare;
      const prepare = new Promise((resolve) => { releasePrepare = resolve; });
      let load;
      let update;
      let ordinary;
      let firstPrepare = true;
      app.hot.addStatusHandler((status) => {
        if (status === 'prepare' && firstPrepare && scenario.startsWith('prepare-')) {
          firstPrepare = false;
          load = app.load();
          load.catch(() => {});
          if (scenario === 'prepare-throw') throw new Error('prepare failed');
          return Promise.reject(new Error('prepare failed'));
        }
        if (status === 'prepare' && firstPrepare && scenario === 'async-prepare') {
          firstPrepare = false;
          load = app.load();
          return load.then(() => prepare);
        }
        if (status === 'ready' && scenario === 'ready-throw') throw new Error('ready failed');
        if (status === 'ready' && scenario === 'ready-reject') return Promise.reject(new Error('ready failed'));
        if (status === 'ready' && scenario === 'ready-waits-import') return load;
      });
      try {
        update = app.hot.check(scenario !== 'ready-waits-import');
        update.catch(() => {});
        if (scenario === 'manifest-error') {
          await assert.rejects(wait(update), /manifest failed/);
          assert.equal(app.hot.status(), 'fail');
          return;
        }
        if (scenario.startsWith('prepare-')) {
          ordinary = await take(/^page\.js$/);
          await assert.rejects(wait(update), /prepare failed/);
          assert.equal(app.hot.status(), 'fail');
          ordinary.deliver(); ordinary = undefined;
          await wait(load);
          assert.equal(app.hot.status(), 'fail', 'late downloads must not revive failed preparation');
          return;
        }
        if (scenario === 'async-prepare') {
          ordinary = await take(/^page\.js$/);
          ordinary.deliver(); ordinary = undefined;
          await wait(load);
          await new Promise((resolve) => setImmediate(resolve));
          assert.equal(app.hot.status(), 'prepare');
          assert.equal(pending.length, 0, 'update downloads must wait for the prepare handler');
          releasePrepare();
        }
        const main = await take(/^main\..*\.hot-update\.js$/);
        if (scenario === 'update-error') {
          main.fail();
          await assert.rejects(wait(update), /Loading hot update chunk/);
          assert.equal(app.hot.status(), 'fail');
          return;
        }
        if (scenario !== 'async-prepare') {
          load = app.load();
          load.catch(() => {});
          ordinary = await take(/^page\.js$/);
        }
        const pageUpdate = await take(/^page\..*\.hot-update\.js$/);
        if (scenario === 'dynamic-update-error') pageUpdate.fail();
        else pageUpdate.deliver();
        main.deliver();
        await new Promise((resolve) => setImmediate(resolve));
        if (ordinary) {
          if (scenario === 'ordinary-load-error') ordinary.fail();
          else ordinary.deliver();
          ordinary = undefined;
        }
        if (scenario === 'dynamic-update-error') {
          await assert.rejects(wait(load), /Loading hot update chunk/);
          await assert.rejects(wait(update), /Loading hot update chunk/);
          assert.equal(statuses.includes('ready'), false);
          assert.equal(app.hot.status(), 'fail');
        } else if (scenario === 'ready-throw' || scenario === 'ready-reject') {
          await assert.rejects(wait(update), /ready failed/);
          assert.equal(app.hot.status(), 'fail');
        } else {
          if (scenario === 'ordinary-load-error') await assert.rejects(wait(load), /Loading chunk/);
          else await wait(load);
          await wait(update);
          if (scenario === 'ready-waits-import') {
            assert.equal(app.hot.status(), 'ready');
            await wait(app.hot.apply());
          }
          assert.equal(app.hot.status(), 'idle');
          assert.equal(app.value(), 2);
        }
      } finally {
        releasePrepare();
        ordinary?.fail();
        for (const item of pending.splice(0)) item.fail();
        load?.catch(() => {});
        update?.catch(() => {});
      }
    },
  })),
);
