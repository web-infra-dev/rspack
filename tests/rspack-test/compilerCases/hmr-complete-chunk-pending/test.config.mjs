import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { rspack } from '@rspack/core';

export default {
  description: 'prepares complete dependencies when an ordinary chunk request is pending and a delta exists',
  options(context) {
    return {
      context: context.getDist('input'),
      entry: './index.js',
      mode: 'development',
      target: 'web',
      devtool: false,
      output: { path: context.getDist('output'), filename: '[name].js', chunkFilename: '[name].js', publicPath: '/' },
      optimization: {
        moduleIds: 'named', chunkIds: 'named', usedExports: false, concatenateModules: false,
        splitChunks: { cacheGroups: {
          default: false, defaultVendors: false,
          shared: { test: /[\\/](dep|extra)\.js$/, name: 'shared', chunks: 'all', enforce: true }
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
      require('./page');
      globalThis.loadOther = () => import(/* webpackChunkName: 'other' */ './other');
      module.hot.accept();
    `);
    write('dep.js', 'module.exports = 2;');
    write('extra.js', 'module.exports = 3;');
    write('other.js', "module.exports = require('./dep');");
    const builds = [];
    for (let step = 0; step < 2; step++) {
      write('page.js', `globalThis.pageValue = ${step ? "require('./dep') + require('./extra')" : '0'}; module.hot.accept();`);
      const stats = await new Promise((resolve, reject) => compiler.run(
        (error, stats) => error ? reject(error) : resolve(stats),
        { modifiedFiles: new Set([path.join(input, 'page.js')]) }
      ));
      assert.equal(stats.hasErrors(), false, stats.toString());
      builds.push(Object.fromEntries(stats.compilation.getAssets().map(asset => [asset.name, asset.source.source().toString()])));
    }
    for (const mode of ['none', 'prefetched', 'complete-before-check', 'pending']) {
      const sandbox = vm.createContext({ self: {}, console, Promise, setTimeout, clearTimeout });
      vm.runInContext(builds[0]['main.js'], sandbox);
      const { req, hot } = sandbox;
      let release;
      const requests = [];
      req.l = (url, done) => {
        const name = path.basename(url.split('?')[0]);
        requests.push(name);
        if (!name.includes('.hot-update.')) {
          const execute = () => {
            vm.runInContext(builds[0][name], sandbox);
            done({ type: 'load' });
          };
          if (name === 'shared.js') release = execute;
          else execute();
          return;
        }
        vm.runInContext(builds[1][name], sandbox);
        done({ type: 'load' });
      };
      // Start with V1's exact bytes before check, without mixing server versions.
      if (mode === 'prefetched') req.hmrS_jsonp.shared = null;
      const request = mode === 'pending' || mode === 'complete-before-check' ? sandbox.loadOther() : undefined;
      if (mode === 'complete-before-check') {
        release();
        release = undefined;
        await request;
      }
      req.hmrM = async () => JSON.parse(builds[1][`main.${req.h()}.hot-update.json`]);
      const manifest = await req.hmrM();
      assert(manifest.c.includes('shared'));
      assert.deepEqual(manifest.load.entries.main, ['shared']);
      await hot.check(false);
      assert.equal(hot.status(), 'ready');
      assert.equal(requests.some(name => name.endsWith('.full.js')), mode !== 'complete-before-check');
      try {
        await hot.apply();
        assert.equal(sandbox.pageValue, 5);
      } finally {
        release?.();
        await request;
      }
    }
  }
};
