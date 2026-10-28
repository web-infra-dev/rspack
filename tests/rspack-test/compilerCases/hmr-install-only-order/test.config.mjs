import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { rspack } from '@rspack/core';

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description:
    'keeps an ordinary hot update when an install-only factory arrives later',
  options(context) {
    return {
      context: context.getDist('input'),
      entry: { a: './a.js', b: './b.js' },
      mode: 'development',
      devtool: false,
      target: 'web',
      output: {
        path: context.getDist('output'),
        filename: '[name].js',
        chunkFilename: '[name].js',
        publicPath: '/',
      },
      optimization: {
        moduleIds: 'named',
        chunkIds: 'named',
        usedExports: true,
        sideEffects: true,
        innerGraph: true,
        concatenateModules: false,
        minimize: false,
        splitChunks: {
          cacheGroups: {
            default: false,
            defaultVendors: false,
            c: {
              test: /[\\/]m\.js$/,
              name: 'c',
              chunks: chunk => ['z', 'x'].includes(chunk.name),
              enforce: true,
              minSize: 0,
            },
          },
        },
      },
      plugins: [new rspack.HotModuleReplacementPlugin()],
    };
  },
  async build(context, compiler) {
    const input = context.getDist('input');
    fs.mkdirSync(input, { recursive: true });
    const sources = {
      'a.js': `
        globalThis.req = __webpack_require__;
        globalThis.loadZ = () => import(/* webpackChunkName: 'z', webpackExports: ['a'] */ './m');
        globalThis.loadX = () => import(/* webpackChunkName: 'x' */ './x');
        globalThis.loadY = () => import(/* webpackChunkName: 'y', webpackExports: ['runA'] */ './y').then(m => m.runA());
        module.hot.accept();
      `,
      'b.js': `
        globalThis.loadY = () => import(/* webpackChunkName: 'y', webpackExports: ['runB'] */ './y').then(m => m.runB());
        module.hot.accept();
      `,
      'm.js': 'export const a = 1; export const b = 2;',
      'x.js': 'export const x = 0;',
      'y.js': `
        import { a, b } from './m';
        export function runA() { return a; }
        export function runB() { return b; }
      `,
    };
    for (const [name, source] of Object.entries(sources)) {
      fs.writeFileSync(path.join(input, name), source);
    }
    const builds = [];
    for (let step = 0; step < 2; step++) {
      if (step) {
        // B drops an export, changing M's factory in shared chunk Y. X starts
        // using M, so A also receives its unchanged factory through chunk C.
        fs.writeFileSync(
          path.join(input, 'b.js'),
          sources['b.js'].replaceAll('runB', 'runA'),
        );
        fs.writeFileSync(
          path.join(input, 'x.js'),
          'import { a } from "./m"; export const x = a;',
        );
      }
      const stats = await new Promise((resolve, reject) => {
        compiler.run(
          (error, stats) => error ? reject(error) : resolve(stats),
          {
            modifiedFiles: step
              ? new Set(['b.js', 'x.js'].map(name => path.join(input, name)))
              : undefined,
          },
        );
      });
      assert.equal(stats.hasErrors(), false, stats.toString());
      builds.push(Object.fromEntries(
        stats.compilation.getAssets().map(asset => [
          asset.name, asset.source.source().toString(),
        ]),
      ));
    }

    for (const order of [['a', 'x', 'y', 'c'], ['a', 'x', 'c', 'y']]) {
      const sandbox = vm.createContext({
        self: {}, console, Promise, setTimeout, clearTimeout,
      });
      vm.runInContext(builds[0]['a.js'], sandbox);
      const req = sandbox.req;
      const queued = new Map();
      let allRequested;
      const ready = new Promise(resolve => {
        allRequested = resolve;
      });
      req.l = (url, done) => {
        const name = path.basename(url);
        if (name.includes('.hot-update.')) {
          queued.set(name.split('.')[0], { name, done });
          if (queued.size === order.length) allRequested();
        } else {
          vm.runInContext(builds[0][name], sandbox);
          done({ type: 'load' });
        }
      };
      await sandbox.loadX();
      assert.equal(await sandbox.loadY(), 1);
      let disposed = 0;
      req.c['./m.js'].hot.dispose(() => {
        disposed++;
      });
      assert.equal(req('./m.js').b, 2);
      req.hmrM = async () => JSON.parse(builds[1][`a.${req.h()}.hot-update.json`]);
      const hot = req.c['./a.js'].hot;
      const update = hot.check(true);
      await ready;
      for (const id of order) {
        const { name, done } = queued.get(id);
        vm.runInContext(builds[1][name], sandbox);
        done({ type: 'load' });
      }
      const updated = await update;
      assert.equal(disposed, 1, `response order: ${order.join(', ')}`);
      assert.deepEqual(Object.keys(sandbox.req('./m.js')), ['a']);
      assert(updated.includes('./m.js'));
      assert.equal(hot.status(), 'idle');
    }
  },
};
