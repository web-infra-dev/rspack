const fs = require('node:fs');
const path = require('node:path');
const { rspack } = require('@rspack/core');

const run = (compiler, changed = []) => new Promise((resolve, reject) => compiler.run((error, stats) => {
  if (error) return reject(error);
  if (stats.hasErrors()) return reject(new Error(stats.toString({ all: false, errors: true })));
  resolve(stats);
}, { modifiedFiles: new Set(changed) }));
const close = compiler => new Promise((resolve, reject) => compiler.close(error => error ? reject(error) : resolve()));
const sources = stats => Object.fromEntries(stats.compilation.getAssets().filter(asset => asset.name.endsWith('.js')).map(asset => [asset.name, asset.source.source().toString()]));

module.exports = [false, true].flatMap(newCache => [false, true].map(concatenateModules => {
  let root, options;
  const write = (file, source) => { const dest = path.join(root, file); fs.writeFileSync(dest, source); return dest; };
  return {
    description: `chunk array cache new=${newCache} concatenate=${concatenateModules}`,
    options(context) {
      root = context.getDist(`chunk-array-cache-${newCache}-${concatenateModules}`);
      fs.rmSync(root, { recursive: true, force: true });
      fs.mkdirSync(root, { recursive: true });
      write('index.js', 'export { load } from "./calls";');
      write('calls.js', 'export const load = () => import("./lazy");');
      write('lazy.js', 'import a from "./a"; import b from "./b"; export default [a,b];');
      for (const name of ['a', 'b', 'c']) write(`${name}.js`, `export default globalThis.${name} || '${name}';`);
      options = {
        context: root, mode: 'production', target: 'node', entry: './index.js', devtool: false,
        cache: { type: 'persistent', storage: { type: 'filesystem', directory: path.join(root, 'cache') } },
        incremental: { modulesCodegen: false },
        experiments: { chunkArrayLoading: true, newCache },
        output: { path: path.join(root, 'dist'), filename: '[name].[contenthash].js', chunkFilename: '[name].[contenthash].js', library: { type: 'commonjs2' } },
        optimization: { minimize: false, concatenateModules, splitChunks: { minSize: 0, cacheGroups: Object.fromEntries(['a', 'b', 'c'].map(name => [name, { test: new RegExp(`[/\\\\]${name}\\.js$`), name, enforce: true, chunks: 'async' }])) } },
      };
      return options;
    },
    compiler(context, compiler) { compiler.outputFileSystem = fs; },
    async build(context, compiler) {
      const first = sources(await run(compiler));
      expect(Object.values(first).join('\n')).toContain('Array.isArray(');
      const warm = await run(compiler, [path.join(root, 'calls.js')]);
      expect(sources(warm)).toEqual(first);
      const cacheLog = warm.toJson({ all: false, logging: 'verbose' }).logging['rspack.Compilation'].entries.find(entry => entry.message?.startsWith('module code generation cache:'));
      expect(Number(cacheLog.message.match(/\((\d+)\/\d+\)/)[1])).toBeGreaterThan(0);
      const lazy = write('lazy.js', 'import a from "./a"; import b from "./b"; import c from "./c"; export default [a,b,c];');
      const changed = await run(compiler, [lazy]);
      const updated = sources(changed);
      expect(updated).not.toEqual(first);
      const main = changed.toJson({ all: false, entrypoints: true }).entrypoints.main.assets[0].name;
      expect((await require(path.join(root, 'dist', main)).load()).default).toEqual(['a', 'b', 'c']);
      await close(compiler);
      compiler.close = callback => callback();
      const restored = rspack(options);
      try { expect(sources(await run(restored))).toEqual(updated); } finally { await close(restored); }
      const disabled = rspack({ ...options, experiments: { ...options.experiments, chunkArrayLoading: false } });
      try {
        const inline = Object.values(sources(await run(disabled))).join('\n');
        expect(inline).not.toContain('Array.isArray(');
        expect(inline).toContain('Promise.all(');
      } finally { await close(disabled); }
    },
  };
}));
