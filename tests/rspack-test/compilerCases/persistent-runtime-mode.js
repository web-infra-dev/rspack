const fs = require('node:fs');
const path = require('node:path');
const { execFileSync } = require('node:child_process');

const modes = [
  { runtimeMode: 'webpack', library: 'modern-module' },
  { runtimeMode: 'rspack', library: 'modern-module' },
  { runtimeMode: 'rspack', library: 'module' },
  { runtimeMode: 'rspack', library: 'modern-module' },
  { runtimeMode: 'webpack', library: 'modern-module' },
];

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [false, true].map((newCache) => ({
  description: `should invalidate persistent modules when the runtime render mode changes (newCache: ${newCache})`,
  options(context) {
    return {
      context: path.resolve(__dirname, '../fixtures/persistent-runtime-mode'),
      mode: 'production',
      target: 'node22',
      entry: './index.js',
      devtool: false,
      incremental: false,
      cache: {
        type: 'persistent',
        storage: {
          type: 'filesystem',
          directory: context.getDist(newCache ? 'new-cache' : 'legacy-cache'),
        },
      },
      experiments: { newCache, outputModule: true, runtimeMode: 'webpack' },
      output: {
        path: context.getDist('output'),
        filename: 'bundle.mjs',
        library: { type: 'modern-module' },
      },
      optimization: { minimize: false },
    };
  },
  async build(context) {
    const manager = context.getCompiler();
    const baseOptions = manager.getOptions();
    await manager.close();

    for (const { runtimeMode, library } of modes) {
      // A fresh compiler with the same mode must reuse the cache, while each
      // transition must rebuild even though the source files did not change.
      for (const warm of [false, true]) {
        manager.setOptions({
          ...baseOptions,
          experiments: { ...baseOptions.experiments, runtimeMode },
          output: { ...baseOptions.output, library: { type: library } },
        });
        const compiler = manager.createCompiler();
        compiler.outputFileSystem = fs;
        const built = [];
        compiler.hooks.compilation.tap('RuntimeModeCacheTest', (compilation) => {
          compilation.hooks.buildModule.tap('RuntimeModeCacheTest', (module) => {
            built.push(path.basename(module.resource));
          });
        });
        const stats = await manager.build();
        expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
        expect(built.sort()).toEqual(warm ? [] : ['engines.cjs', 'index.js']);
        expect(
          execFileSync(process.execPath, [context.getDist('output/bundle.mjs')], {
            encoding: 'utf8',
          }).trim(),
        ).toBe('{"yaml":1}');
        await manager.close();
      }
    }
  },
}));
