import assert from 'node:assert/strict';
import { sep } from 'node:path';
import { NormalModule, optimize } from '@rspack/core';

let captureChunks;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description: 'updates compilation collection arrays in place when membership changes',
  options: () => ({
    mode: 'development',
    entry: './entry.js',
    optimization: {
      concatenateModules: false,
      splitChunks: false,
    },
    plugins: [
      new optimize.SplitChunksPlugin({
        cacheGroups: {
          observe: {
            test() {
              captureChunks();
              return false;
            },
          },
        },
      }),
      new optimize.LimitChunkCountPlugin({ maxChunks: 1 }),
    ],
    module: { rules: [{ test: /entry\.js$/, use: './loader.cjs' }] },
  }),
  compiler(_context, compiler) {
    compiler.hooks.thisCompilation.tap('CollectionSnapshots', (compilation) => {
      const inner = compilation.__internal_getInner();
      const names = ['modules', 'builtModules', 'chunks'];
      const collections = names.map((name) => {
        const binding = inner[name];
        const array = binding.values();
        assert.deepEqual(array, []);
        assert.equal(binding.values(), array);
        return { name, binding, array, iterator: compilation[name].values() };
      });
      const chunks = collections.find(({ name }) => name === 'chunks');
      let sawMultipleChunks = false;
      captureChunks = () => {
        // SplitChunks runs before LimitChunkCount merges the async chunk.
        assert.equal(chunks.binding.values(), chunks.array);
        if (chunks.array.length > 1) {
          sawMultipleChunks = true;
        }
      };
      let source = 'import "./a.js"; export default 1;';
      NormalModule.getCompilationHooks(compilation).loader.tap(
        'CollectionSnapshots',
        (context) => {
          context.collectionSource = source;
          // Even an already cached empty array must not bypass the borrowed
          // module graph guard while make/rebuild is running.
          for (const { name, binding } of collections) {
            if (name !== 'chunks') {
              assert.throws(() => binding.values(), /ModuleGraph is not available/);
            }
          }
        },
      );
      const refresh = ({ name, binding, array }) => {
        assert.equal(binding.values(), array, `${name} must retain its array`);
        assert.equal(binding.values(), array, `${name} must reuse an unchanged array`);
        assert.equal(array.length, binding.size());
        assert.equal(Object.keys(array).length, array.length);
        assert.deepEqual([...compilation[name]], array);
        for (const value of array) {
          assert(binding.has(value));
        }
      };
      compilation.hooks.finishModules.tapPromise('CollectionSnapshots', async () => {
        const modules = collections.filter(({ name }) => name !== 'chunks');
        for (const state of modules) {
          refresh(state);
          assert.equal(state.array.length, 2);
          // An iterator created over the empty array observes its in-place growth.
          assert.deepEqual([...state.iterator], state.array);
        }
        const entry = [...compilation.modules].find((module) =>
          module.identifier().endsWith(`${sep}entry.js`),
        );
        assert(entry);
        const rebuild = () => new Promise((resolve, reject) => {
          compilation.rebuildModule(entry, (error) => {
            if (error) {
              reject(error);
            } else {
              resolve();
            }
          });
        });
        // Replace a dependency without changing the collection size.
        source = 'import "./b.js"; export default 1;';
        await rebuild();
        for (const state of modules) {
          refresh(state);
          assert.equal(state.array.length, 2);
          assert(state.array.some((module) => module.identifier().endsWith(`${sep}b.js`)));
          assert(!state.array.some((module) => module.identifier().endsWith(`${sep}a.js`)));
        }
        // Removing the dependency must also truncate the existing array.
        source = 'export default 1;';
        await rebuild();
        for (const state of modules) {
          refresh(state);
          assert.equal(state.array.length, 1);
          assert.equal(state.array[0], entry);
          assert.equal(state.array[1], undefined);
        }
        // Build an async chunk so the chunk cache also exercises shrinkage.
        source = 'export default import("./b.js");';
        await rebuild();
      });
      compilation.hooks.processAssets.tap('CollectionSnapshots', () => {
        for (const state of collections) {
          refresh(state);
        }
        assert(sawMultipleChunks);
        assert.equal(chunks.array.length, 1);
        assert.equal(chunks.array[1], undefined);
        assert.deepEqual([...chunks.iterator], chunks.array);
      });
    });
  },
};
