import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { rspack } from '@rspack/core';

export default ['webpack', 'rspack'].map((runtimeMode) => ({
  name: runtimeMode,
  description: 'preserves installed chunks when replacing the loading runtime',
  options(context) {
    return {
      context: context.getDist('input'),
      entry: './index.js',
      mode: 'development',
      devtool: false,
      target: 'web',
      experiments: { runtimeMode },
      output: {
        path: context.getDist('output'),
        filename: '[name].js',
        chunkFilename: '[name].js',
        publicPath: '/',
      },
      optimization: { moduleIds: 'named', chunkIds: 'named', splitChunks: false },
      plugins: [new rspack.HotModuleReplacementPlugin()],
    };
  },
  async build(context, compiler) {
    const input = context.getDist('input');
    fs.mkdirSync(input, { recursive: true });
    const assets = {};
    const build = async (files) => {
      for (const [name, source] of Object.entries(files))
        fs.writeFileSync(path.join(input, name), source);
      const stats = await new Promise((resolve, reject) =>
        compiler.run(
          (error, stats) => (error ? reject(error) : resolve(stats)),
          {
            modifiedFiles: new Set(
              Object.keys(files).map((name) => path.join(input, name)),
            ),
          },
        ),
      );
      assert.equal(stats.hasErrors(), false, stats.toString());
      for (const asset of stats.compilation.getAssets())
        assets[asset.name] = asset.source.source().toString();
      return stats.hash;
    };
    const sandbox = vm.createContext({
      console,
      Promise,
      setTimeout,
      clearTimeout,
    });
    sandbox.self = sandbox;
    const requests = [];
    const scripts = [];
    const head = {
      appendChild(script) {
        scripts.push(script);
        script.parentNode = head;
        const name = path.basename(script.src);
        requests.push(name);
        vm.runInContext(assets[name], sandbox, { filename: name });
        queueMicrotask(() => script.onload?.({ type: 'load', target: script }));
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
    sandbox.fetch = async (url) => {
      const body = assets[path.basename(url)];
      return {
        status: body === undefined ? 404 : 200,
        ok: body !== undefined,
        json: async () => JSON.parse(body),
      };
    };
    const hash = await build({
      'index.js': `
        globalThis.app = { hot: module.hot,
          load: () => import(/* webpackChunkName: "page" */ './page') };
        require('./trigger');
        module.hot.accept('./trigger', () => require('./trigger'));
      `,
      'trigger.js': 'module.exports = 0;',
      'page.js': 'module.exports = 1; module.hot.accept();',
      'second.js': 'module.exports = 10;',
    });
    vm.runInContext(assets['main.js'], sandbox);
    const { app } = sandbox;
    assert.equal((await app.load()).default, 1);

    // Adding fetchPriority changes the chunk loading runtime's generated source.
    await build({
      'trigger.js': `module.exports = () => import(
        /* webpackChunkName: "second", webpackFetchPriority: "high" */ './second'
      );`,
      'page.js': 'module.exports = 2; module.hot.accept();',
    });
    assert.match(
      assets[`main.${hash}.hot-update.js`],
      /runtime\/jsonp_chunk_loading/,
    );
    await app.hot.check(true);
    assert.equal(
      (await app.load()).default,
      2,
      'must not replay the old page factory',
    );
    assert.equal(requests.filter((name) => name === 'page.js').length, 1);
    assert.equal(app.hot.status(), 'idle');
  },
}));
