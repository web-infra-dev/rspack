const fs = require('node:fs');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
const { HotModuleReplacementPlugin, RuntimeGlobals, container } = require('@rspack/core');

const variants = [
  {},
  { concatenate: true },
  { runtimeMode: 'rspack', concatenate: true },
  { arrow: false, named: true },
  { esm: true, runtimeMode: 'rspack' },
  { modern: true, runtimeMode: 'rspack', kind: 'ensure' },
  { kind: 'ensure' },
  { kind: 'amd' },
  { kind: 'lazy-once' },
  { kind: 'lazy' },
  { kind: 'container' },
  { mode: 'development' },
  { hmr: true },
  { enabled: false },
  { onlySingle: true },
  { minimize: true },
  { priority: false },
  { multi: true },
];

module.exports = variants.map((variant, index) => {
  let root;
  const active = variant.enabled !== false && variant.mode !== 'development' && !variant.hmr && !variant.onlySingle && variant.kind !== 'lazy';
  const esm = variant.esm || variant.modern;
  return {
    description: `chunk array loading ${JSON.stringify(variant)}`,
    options(context) {
      root = context.getDist(`chunk-array-${index}`);
      fs.rmSync(root, { recursive: true, force: true });
      fs.mkdirSync(path.join(root, 'context'), { recursive: true });
      const write = (name, source) => fs.writeFileSync(path.join(root, name), source);
      write('a.js', 'export default globalThis.chunkArrayA || "a";');
      write('b.js', 'export default globalThis.chunkArrayB || "b";');
      write('context/lazy.js', 'import a from "../a"; import b from "../b"; export default [a,b];');
      write('high.js', 'import a from "./a"; import b from "./b"; export default [a,b];');
      write('single.js', 'export default "single";');
      write('other.js', 'export default 42;');
      const load = variant.onlySingle ? 'import("./single")'
        : variant.kind === 'ensure' ? 'new Promise((resolve, reject) => require.ensure([], require => resolve(require("./context/lazy")), error => reject(error), "group"))'
        : variant.kind === 'amd' ? 'new Promise(resolve => require(["./context/lazy"], resolve))'
        : variant.kind?.startsWith('lazy') ? `require.context('./context', false, /lazy/, '${variant.kind}')('./lazy.js')`
        : 'import("./context/lazy")';
      write('index.js', `export const load = () => ${load};
        export const single = () => import('./single');
        export const local = () => import('./index');
        export const getEnsure = () => __webpack_chunk_load__;
        export const setEnsure = next => { __webpack_chunk_load__ = next; };
        ${active && variant.priority !== false ? 'export const high = () => import(/* webpackFetchPriority: "high" */ "./high");' : ''}`);
      return {
        context: root,
        mode: variant.mode || 'production',
        target: 'node',
        entry: variant.kind === 'container' ? {} : variant.multi ? { main: './index.js', other: './other.js' } : './index.js',
        amd: {},
        devtool: 'source-map',
        experiments: { chunkArrayLoading: variant.enabled !== false, runtimeMode: variant.runtimeMode || 'webpack', outputModule: !!esm },
        plugins: [
          ...(variant.hmr ? [new HotModuleReplacementPlugin()] : []),
          ...(variant.kind === 'container' ? [new container.ContainerPlugin({ name: 'remote', filename: 'remote.js', library: { type: 'commonjs2' }, exposes: { './lazy': './context/lazy' } })] : []),
        ],
        output: { path: path.join(root, 'dist'), filename: esm ? '[name].mjs' : '[name].js', chunkFilename: esm ? '[id].mjs' : '[id].js', module: !!esm, library: { type: variant.modern ? 'modern-module' : esm ? 'module' : 'commonjs2' }, environment: { arrowFunction: variant.arrow !== false } },
        optimization: {
          minimize: !!variant.minimize,
          concatenateModules: !!variant.concatenate,
          chunkIds: variant.named ? 'named' : 'deterministic',
          runtimeChunk: variant.multi ? { name: entry => entry.name + '-runtime' } : false,
          splitChunks: { minSize: 0, cacheGroups: Object.fromEntries(['a', 'b'].map(name => [name, { test: new RegExp(`[/\\\\]${name}\\.js$`), name: `shared-${name}`, enforce: true, chunks: 'async' }])) },
        },
      };
    },
    compiler(context, compiler) { compiler.outputFileSystem = fs; },
    async check() {
      const output = path.join(root, 'dist');
      const sources = fs.readdirSync(output).filter(name => /\.(m?js)$/.test(name)).map(name => fs.readFileSync(path.join(output, name), 'utf8')).join('\n');
      expect(sources.includes('Array.isArray(')).toBe(active);
      expect(sources).not.toContain('__webpack_require__.q');
      expect(sources).not.toContain('chunkGroups');
      expect(RuntimeGlobals.ensureChunk).toBe('__webpack_require__.e');
      expect(RuntimeGlobals.ensureChunkGroup).toBeUndefined();
      expect(fs.readdirSync(output).some(name => name.endsWith('.map'))).toBe(true);
      if (variant.multi) expect(fs.readFileSync(path.join(output, 'other-runtime.js'), 'utf8')).not.toContain('Array.isArray(');
      if (variant.kind === 'container') {
        const remote = require(path.join(output, 'remote.js'));
        const factory = await remote.get('./lazy');
        expect(factory().default).toEqual(['a', 'b']);
        return;
      }
      const main = path.join(output, esm ? 'main.mjs' : 'main.js');
      const exports = esm ? await import(pathToFileURL(main).href) : require(main);
      expect((await exports.single()).default).toBe('single');
      expect((await exports.local()).load).toBe(exports.load);
      const first = exports.load();
      const second = exports.load();
      expect(first).not.toBe(second);
      for (const loaded of await Promise.all([first, second])) expect(loaded.default).toEqual(variant.onlySingle ? 'single' : ['a', 'b']);
      if (!active) return;
      // Modern ESM imports are immutable bindings, so replacing the loader is unsupported.
      if (variant.modern) { expect((await exports.high()).default).toEqual(['a', 'b']); return; }

      const original = exports.getEnsure();
      const calls = [];
      exports.setEnsure(function (ids, priority) {
        calls.push({ ids, priority, length: arguments.length, receiver: this });
        return original.apply(this, arguments);
      });
      try {
        await exports.load();
        const ids = calls[0].ids;
        expect(Array.isArray(ids)).toBe(true);
        expect(ids).toHaveLength(3);
        expect(calls.slice(1).map(call => call.ids)).toEqual(ids);
        expect(calls.every(call => call.length === 1)).toBe(true);
        expect(calls.every(call => call.receiver === calls[0].receiver)).toBe(true);
        if (variant.priority !== false) {
          calls.length = 0;
          await exports.high();
          expect(calls).toHaveLength(4);
          expect(calls.every(call => call.length === 2 && call.priority === 'high')).toBe(true);
        }

        // Scalar results remain nested and ordered; duplicate and string IDs are not removed.
        exports.setEnsure(id => Promise.resolve([id]));
        const ordered = ['a', 2, 'a'];
        const one = original(ordered);
        const two = original(ordered);
        expect(one).not.toBe(two);
        expect(await one).toEqual([['a'], [2], ['a']]);
        expect(await two).toEqual(await one);
        expect(await original([])).toEqual([]);
        const error = new Error('chunk failed');
        let attempts = 0;
        exports.setEnsure(() => { if (++attempts === 2) throw error; return Promise.resolve(); });
        expect(() => original(ordered)).toThrow(error);
        expect(attempts).toBe(2);
        exports.setEnsure(() => Promise.reject(error));
        await expect(original(ordered)).rejects.toBe(error);
      } finally {
        exports.setEnsure(original);
      }
      expect((await exports.load()).default).toEqual(['a', 'b']);
    },
  };
});
