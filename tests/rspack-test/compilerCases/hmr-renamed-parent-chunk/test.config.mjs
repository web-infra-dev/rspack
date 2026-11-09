import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { rspack } from '@rspack/core';

export default ['webpack', 'rspack'].map(runtimeMode => ({
  description: `restores a dependency whose parent chunk was renamed (${runtimeMode})`,
  options(context) {
    return {
      context: context.getDist('input'),
      entry: './index.js', mode: 'development', target: 'web', devtool: false,
      experiments: { runtimeMode },
      output: { path: context.getDist('output'), filename: '[name].js', chunkFilename: '[name].js', publicPath: '/' },
      optimization: { moduleIds: 'named', chunkIds: 'named', concatenateModules: false },
      plugins: [new rspack.HotModuleReplacementPlugin()]
    };
  },
  async build(context, compiler) {
    const input = context.getDist('input');
    fs.mkdirSync(input, { recursive: true });
    const write = (name, source) => fs.writeFileSync(path.join(input, name), source);
    write('a.js', `require('./x'); globalThis.loadB = () => import(/* webpackChunkName: 'b' */ './b');`);
    write('b.js', `globalThis.readX = () => require('./x');`);
    write('x.js', 'module.exports = 1;');
    const builds = [];
    for (const name of ['a', 'renamed']) {
      write('index.js', `
        globalThis.hot = module.hot; globalThis.getHash = () => __webpack_hash__;
        globalThis.loadA = () => import(/* webpackChunkName: '${name}' */ './a');
        module.hot.accept();
      `);
      const stats = await new Promise((resolve, reject) => compiler.run(
        (error, stats) => error ? reject(error) : resolve(stats),
        { modifiedFiles: new Set([path.join(input, 'index.js')]) }
      ));
      assert.equal(stats.hasErrors(), false, stats.toString());
      builds.push(Object.fromEntries(stats.compilation.getAssets().map(asset => [asset.name, asset.source.source().toString()])));
    }
    let assets = builds[0];
    const sandbox = vm.createContext({ self: {}, console, Promise, setTimeout, clearTimeout });
    vm.runInContext(assets['main.js'], sandbox);
    const requests = [];
    sandbox.document = {
      getElementsByTagName: () => [],
      createElement: () => ({ setAttribute() {} }),
      head: {
        appendChild(script) {
          const name = path.basename(script.src);
          requests.push(name);
          assert(assets[name], `missing asset ${name}`);
          queueMicrotask(() => {
            vm.runInContext(assets[name], sandbox);
            script.onload({ type: 'load', target: script });
          });
        }
      }
    };
    sandbox.fetch = async url => ({ ok: true, json: async () => JSON.parse(assets[path.basename(url)]) });
    await sandbox.loadA();
    await sandbox.loadB();
    assert.equal(sandbox.readX(), 1);
    assets = builds[1];
    const manifest = JSON.parse(assets[`main.${sandbox.getHash()}.hot-update.json`]);
    assert.deepEqual(manifest.r, ['a']);
    assert.deepEqual(manifest.load, { b: ['renamed'] });
    requests.length = 0;
    await sandbox.hot.check(true);
    assert(requests.some(name => name.startsWith('renamed.') && name.endsWith('.full.js')));
    assert.equal(sandbox.readX(), 1, 'the loaded child chunk still needs x from the renamed parent chunk');
    assert.equal(sandbox.hot.status(), 'idle');
  }
}));
