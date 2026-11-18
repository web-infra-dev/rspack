import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { rspack } from '@rspack/core';

export default [false, true].flatMap(incremental =>
  ['webpack', 'rspack'].map(runtimeMode => {
    let step;
    return {
      description: `plans loaded async dependencies before disposing modules (${incremental}, ${runtimeMode})`,
      options(context) {
        return {
          context: context.getDist('input'),
          entry: './index.js', mode: 'development', target: 'web', devtool: false,
          incremental, experiments: { runtimeMode },
          output: { path: context.getDist('output'), filename: '[name].js', chunkFilename: '[name].js', publicPath: '/' },
          optimization: {
            moduleIds: 'named', chunkIds: 'named', usedExports: false, concatenateModules: false,
            splitChunks: { cacheGroups: {
              default: false, defaultVendors: false,
              shared: { test: /[\\/]dep\.js$/, name: () => step < 2 ? 'shared' : 'moved', chunks: 'all', minChunks: 2, enforce: true },
              target: { test: /[\\/]target\.js$/, name: () => step < 2 ? 'shared-target' : 'moved-target', chunks: 'all', minChunks: 2, enforce: true }
            } }
          },
          plugins: [new rspack.HotModuleReplacementPlugin()]
        };
      },
      async build(context, compiler) {
        const input = context.getDist('input');
        fs.mkdirSync(input, { recursive: true });
        const write = (name, source) => fs.writeFileSync(path.join(input, name), source);
        write('index.js', `
          globalThis.hot = module.hot;
          globalThis.cache = require.cache; globalThis.getHash = () => __webpack_hash__;
          globalThis.loadPage = () => import(/* webpackChunkName: 'page' */ './page');
          globalThis.loadOther = () => import(/* webpackChunkName: 'other' */ './other');
          globalThis.loadTarget = () => import(/* webpackChunkName: 'target-page' */ './target');
          globalThis.loadKeep = () => import(/* webpackChunkName: 'target-page' */ './keep');
          module.hot.accept();
        `);
        write('keep.js', 'module.exports = 1;');
        write('target.js', 'module.exports = 42;');
        const builds = [];
        for (step = 0; step < 4; step++) {
          write('page.js', `globalThis.readPage = () => ${step === 3 ? '9' : "require('./dep')"}; module.hot.accept();`);
          write('other.js', `${step ? "require('./target');" : ''} module.exports = ${step ? "require('./dep')" : '0'};`);
          write('dep.js', `module.exports = ${step < 2 ? 2 : step + 1};`);
          const stats = await new Promise((resolve, reject) => compiler.run(
            (error, stats) => error ? reject(error) : resolve(stats),
            { modifiedFiles: new Set(['page.js', 'other.js', 'dep.js'].map(name => path.join(input, name))) }
          ));
          assert.equal(stats.hasErrors(), false, stats.toString());
          builds.push(Object.fromEntries(stats.compilation.getAssets().map(asset => [asset.name, asset.source.source().toString()])));
        }
        let assets = builds[0];
        const requests = [];
        function startApp(readScript) {
          const sandbox = vm.createContext({ self: {}, console, Promise, setTimeout, clearTimeout });
          vm.runInContext(builds[0]['main.js'], sandbox);
          sandbox.document = {
            getElementsByTagName: () => [],
            createElement: () => ({ setAttribute() {} }),
            head: {
              appendChild(script) {
                const name = path.basename(script.src);
                requests.push(name);
                const scriptSource = readScript(name);
                assert(scriptSource, `missing asset ${name}`);
                queueMicrotask(() => {
                  vm.runInContext(scriptSource, sandbox);
                  script.onload({ type: 'load', target: script });
                });
              }
            }
          };
          return sandbox;
        }
        const sandbox = startApp(name => assets[name]);
        const { hot, cache } = sandbox;
        sandbox.fetch = async url => ({ ok: true, json: async () => JSON.parse(assets[path.basename(url)]) });
        await sandbox.loadPage();
        assert.equal(sandbox.readPage(), 2);
        const oldModule = cache['./dep.js'];
        let disposed = 0;
        oldModule.hot.dispose(() => disposed++);
        assets = builds[1];
        const manifest = JSON.parse(assets[`main.${sandbox.getHash()}.hot-update.json`]);
        assert(!manifest.changed?.['./dep.js'], 'membership alone must not update the module');
        assert.deepEqual(manifest.load.page, ['shared']);
        requests.length = 0;
        await hot.check(true);
        assert.equal(cache['./dep.js'], oldModule);
        assert.equal(disposed, 0);
        assert(requests.some(name => name.startsWith('shared.') && name.endsWith('.full.js')));
        assert(!requests.some(name => name.startsWith('other.')));
        assets = builds[2];
        requests.length = 0;
        await hot.check(true);
        assert.equal(sandbox.readPage(), 3, 'an unchanged importer needs the moved dependency');
        assert.equal(disposed, 1);
        assert(requests.some(name => name.startsWith('moved.') && name.endsWith('.full.js')));
        assert(!requests.some(name => name.startsWith('other.')));
        const movedModule = cache['./dep.js'];
        movedModule.hot.dispose(() => disposed++);
        assets = builds[3];
        requests.length = 0;
        await hot.check(false);
        assert.equal(cache['./dep.js'], movedModule, 'check must not dispose modules');
        await hot.apply();
        assert.equal(sandbox.readPage(), 9);
        assert.equal(disposed, 2);
        assert.equal(cache['./dep.js'], undefined);
        assert(!requests.some(name => name.startsWith('other.')));
        assert.equal((await sandbox.loadOther()).default, 4);
        assert.equal(hot.status(), 'idle');

        // An old import retries across two updates. Either its dependency or the import target itself moves.
        for (const [chunk, expected, runImport] of [
          ['page', ['shared', 'moved'], async app => { await app.loadPage(); assert.equal(app.readPage(), 3); }],
          ['target-page', ['shared-target', 'moved-target'], async app => assert.equal((await app.loadTarget()).default, 42)]
        ]) {
          let round = 0;
          const responses = [builds[1], builds[0], builds[2]];
          requests.length = 0;
          const retrying = startApp(name => name === `${chunk}.js` ? responses.shift()[name] : builds[round][name]);
          retrying.fetch = async url => {
            const manifest = JSON.parse(builds[++round][path.basename(url)]);
            return { ok: true, json: async () => manifest };
          };
          await runImport(retrying);
          assert.equal(round, 2);
          assert.deepEqual(requests.filter(name => name.endsWith('.full.js')).map(name => name.split('.')[0]), expected);
          assert(!requests.some(name => name.startsWith('other.')));
        }
      }
    };
  })
);
