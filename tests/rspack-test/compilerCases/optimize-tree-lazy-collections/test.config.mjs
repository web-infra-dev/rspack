import assert from 'node:assert/strict';

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  name: 'lazy-collections',
  description: 'shares a lazy readonly set facade for modules, built modules and chunks',
  options: () => ({ mode: 'development', entry: './entry.js' }),
  compiler(_context, compiler) {
    compiler.hooks.thisCompilation.tap('LazyCollections', (compilation) => {
      const inner = compilation.__internal_getInner();
      const collections = ['modules', 'builtModules', 'chunks'].map((name) => {
        const binding = inner[name];
        assert.equal(inner[name], binding);
        const getValues = binding.values;
        const state = { name, binding, reads: 0 };
        Object.defineProperty(inner, name, {
          configurable: true,
          get: () => binding,
        });
        Object.defineProperty(binding, 'values', {
          configurable: true,
          value() {
            state.reads++;
            return getValues.call(binding);
          },
        });
        state.view = compilation[name];
        state.iterator = state.view[Symbol.iterator]();
        assert.equal(state.reads, 0);
        return state;
      });
      assert.equal(new Set(collections.map(({ view }) => view.constructor)).size, 1);
      compilation.hooks.optimizeTree.tap('EmptyHook', () => {});
      compilation.hooks.optimizeTree.tap('LazyCollections', (chunks, modules) => {
        try {
          assert.equal(chunks, compilation.chunks);
          assert.equal(modules, compilation.modules);
          const chunk = compilation.entrypoints.get('main').getEntrypointChunk();
          const [entry] = compilation.chunkGraph.getChunkEntryModulesIterable(chunk);
          assert(entry);
          for (const state of collections) {
            const { name, view, iterator } = state;
            assert.equal(compilation[name], view);
            assert.equal(state.reads, 0);
            const size = view.size;
            assert(size >= (name === 'chunks' ? 1 : 2));
            assert.equal(view.has(name === 'chunks' ? chunk : entry), true);
            assert.equal(view.has({}), false);
            assert.equal(view.has(name === 'chunks' ? entry : chunk), false);
            assert.equal(state.reads, 0);
            const values = [...view.values()];
            assert.equal(state.reads, 1);
            assert.equal(values.length, size);
            assert.deepEqual([...iterator], values);
            assert.equal(state.reads, 2);
            assert.deepEqual([...view.keys()], values);
            assert.deepEqual([...view.entries()], values.map((value) => [value, value]));
            const visited = [];
            const receiver = {};
            view.forEach(function (value, key, set) {
              assert.equal(this, receiver);
              assert.equal(value, key);
              assert.equal(set, view);
              visited.push(value);
            }, receiver);
            assert.deepEqual(visited, values);
            assert.equal(state.reads, 5);
          }
        } finally {
          for (const { name, binding } of collections) {
            delete inner[name];
            delete binding.values;
          }
        }
      });
      compilation.hooks.afterSeal.tap('LazyCollections', () => {
        for (const state of collections) {
          assert.equal(state.reads, 5);
        }
      });
    });
  },
};
