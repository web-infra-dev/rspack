import assert from 'node:assert/strict';

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  name: 'lazy-modules',
  description: 'only converts optimizeTree modules when iterated',
  options: () => ({ mode: 'development', entry: './entry.js' }),
  compiler(_context, compiler) {
    compiler.hooks.thisCompilation.tap('LazyModules', (compilation) => {
      const inner = compilation.__internal_getInner();
      const nativeModules = inner.modules;
      const getValues = nativeModules.values;
      let moduleReads = 0;
      // Keep the instrumented native collection behind the cached facade.
      Object.defineProperty(inner, 'modules', {
        configurable: true,
        get: () => nativeModules,
      });
      Object.defineProperty(nativeModules, 'values', {
        configurable: true,
        value() {
          moduleReads++;
          return getValues.call(nativeModules);
        },
      });

      // Accessing modules before the graph is built must not enumerate it.
      const retained = compilation.modules;
      assert.equal(moduleReads, 0);
      compilation.hooks.optimizeTree.tap('EmptyHook', () => {});
      compilation.hooks.optimizeTree.tap('LazyModules', (_chunks, modules) => {
        try {
          assert.equal(moduleReads, 0);
          assert.equal(modules, retained);
          const size = modules.size;
          assert(size >= 2);
          const chunk = compilation.entrypoints.get('main').getEntrypointChunk();
          const [entry] =
            compilation.chunkGraph.getChunkEntryModulesIterable(chunk);
          assert(entry);
          assert.equal(modules.has(entry), true);
          assert.equal(modules.has({}), false);
          assert.equal(moduleReads, 0);

          const values = [...modules.values()];
          assert.equal(moduleReads, 1);
          assert.equal(values.length, size);
          assert(values.includes(entry));
          assert(
            values.some((module) => module.resource?.endsWith('dependency.js')),
          );
          assert.deepEqual([...modules], values);
          assert.equal(moduleReads, 2);
        } finally {
          delete inner.modules;
          delete nativeModules.values;
        }
      });
      compilation.hooks.afterSeal.tap('LazyModules', () => {
        assert.equal(moduleReads, 2);
      });
    });
  },
};
