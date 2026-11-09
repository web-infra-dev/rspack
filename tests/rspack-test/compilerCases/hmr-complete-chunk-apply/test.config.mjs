import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { rspack } from '@rspack/core';

export default [false, true].map(withDelta => ({
  description: `stages complete entry dependencies (existing delta: ${withDelta})`,
  options(context) {
    return {
      context: context.getDist('input'),
      entry: './index.js', mode: 'development', target: 'web', devtool: false,
      output: { path: context.getDist('output'), filename: '[name].js', chunkFilename: '[name].js', publicPath: '/' },
      optimization: {
        moduleIds: 'named', chunkIds: 'named', usedExports: false, concatenateModules: false,
        splitChunks: { cacheGroups: {
          default: false, defaultVendors: false,
          shared: { test: /[\\/](dep|extra)\.js$/, name: 'shared', chunks: withDelta ? 'all' : 'initial', enforce: true }
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
      globalThis.req = __webpack_require__;
      globalThis.hot = module.hot;
      globalThis.readStable = () => require('./stable');
      require('./page');
      ${withDelta ? "globalThis.loadOther = () => import(/* webpackChunkName: 'other' */ './other');" : ''}
      module.hot.accept();
    `);
    write('dep.js', 'module.exports = 2;');
    write('extra.js', 'module.exports = 3;');
    write('other.js', "module.exports = require('./dep');");
    const builds = [];
    for (let step = 0; step < 2; step++) {
      write('stable.js', `module.exports = ${step + 2}; module.hot.accept();`);
      write('page.js', `globalThis.pageValue = ${step ? "require('./dep') + require('./extra')" : '0'}; module.hot.accept();`);
      const stats = await new Promise((resolve, reject) => compiler.run(
        (error, stats) => error ? reject(error) : resolve(stats),
        { modifiedFiles: new Set(['page.js', 'stable.js'].map(name => path.join(input, name))) }
      ));
      assert.equal(stats.hasErrors(), false, stats.toString());
      builds.push(Object.fromEntries(stats.compilation.getAssets().map(asset => [asset.name, asset.source.source().toString()])));
    }
    const modes = ['none', 'prefetched', 'abort', ...(withDelta ? ['loaded', 'pending', 'apply-pending'] : [])];
    for (const mode of modes) for (const reverse of [false, true]) {
      const sandbox = vm.createContext({ self: {}, console, Promise, setTimeout, clearTimeout });
      vm.runInContext(builds[0]['main.js'], sandbox);
      const { req, hot } = sandbox;
      const updates = [];
      let release;
      let applied = false;
      req.l = (url, done) => {
        const name = path.basename(url.split('?')[0]);
        const update = name.includes('.hot-update.');
        const source = builds[update || applied ? 1 : 0][name];
        const execute = () => {
          vm.runInContext(source, sandbox);
          done({ type: 'load' });
        };
        if (update) updates.push({ name, execute });
        else if (name === 'shared.js') release = execute;
        else execute();
      };
      if (mode === 'prefetched') req.hmrS_jsonp.shared = null;
      let pending = ['pending', 'loaded'].includes(mode) ? sandbox.loadOther() : undefined;
      if (mode === 'loaded') {
        release();
        await pending;
        release = undefined;
      }
      const oldDep = req.c['./dep.js'];
      let depDisposals = 0;
      oldDep?.hot.dispose(() => depDisposals++);
      const oldFactory = req.m['./stable.js'];
      assert.equal(sandbox.readStable(), 2);
      req.hmrM = async () => JSON.parse(builds[1][`main.${req.h()}.hot-update.json`]);
      const manifest = await req.hmrM();
      assert(manifest.c.includes('shared'));
      assert.deepEqual(manifest.load.main, ['shared']);
      const check = hot.check(false);
      await new Promise(setImmediate);
      const shared = updates.filter(({ name }) => name.startsWith('shared.'));
      assert.equal(shared.length, 1, 'full and delta use the same JSONP callback and must download sequentially');
      assert.equal(shared[0].name.endsWith('.full.js'), mode !== 'loaded');
      const processed = new Set();
      for (;;) {
        const batch = updates.filter(update => !processed.has(update));
        if (!batch.length) break;
        if (reverse) batch.reverse();
        for (const update of batch) {
          processed.add(update);
          update.execute();
        }
        await new Promise(setImmediate);
      }
      assert.equal(updates.filter(({ name }) => name.startsWith('shared.')).length, mode === 'loaded' ? 1 : 2);
      await check;
      assert.equal(hot.status(), 'ready');
      assert.equal(sandbox.readStable(), 2);
      assert.equal(req.m['./stable.js'], oldFactory);
      assert.equal(req.m['./extra.js'], undefined);
      if (mode === 'abort') {
        req.c['./page.js'].hot.decline();
        req.c['./page.js'].hot.invalidate();
        await assert.rejects(hot.apply(), /self decline/);
        assert.equal(sandbox.readStable(), 2);
        assert.equal(req.m['./extra.js'], undefined);
        continue;
      }
      applied = true;
      if (mode === 'apply-pending') hot.addStatusHandler(status => {
        if (status === 'dispose') pending = req.e('shared');
      });
      const updated = await hot.apply();
      assert.equal(sandbox.pageValue, 5);
      assert.equal(sandbox.readStable(), 3);
      if (withDelta) assert(!updated.includes('./dep.js'), 'unchanged dependencies must not propagate HMR');
      if (oldDep) assert.equal(req.c['./dep.js'], oldDep);
      assert.equal(depDisposals, 0);
      assert.equal(req.hmrS_jsonp.shared, 0);
      let timer;
      try {
        await Promise.race([
          pending,
          new Promise((_, reject) => { timer = setTimeout(() => reject(new Error('import did not resolve')), 1000); })
        ]);
      } finally {
        clearTimeout(timer);
        release?.();
      }
      assert.equal(hot.status(), 'idle');
    }
  }
}));
