import assert from 'node:assert/strict';
import util from 'node:util';
import { createFsFromVolume, Volume } from 'memfs';
import { rspack } from '@rspack/core';
import { runCompiler, closeCompiler } from '@rspack/test-tools/helper/lifecycle';

const scenarios = [
  'no-hook',
  'empty-sync',
  'empty-promise',
  'empty-async',
  'chunks-only',
  'size-only',
  'has-only',
  'read-modules',
];

const cases = scenarios.map((name) => ({
  name,
  description: `materializes only the optimizeTree collections used by ${name}`,
  options: () => ({ mode: 'development', entry: './entry.js' }),
  compiler(_context, compiler) {
    compiler.hooks.thisCompilation.tap('LazyCollections', (compilation) => {
      const inner = compilation.__internal_getInner();
      const nativeModules = inner.modules;
      const getModules = nativeModules.values;
      let moduleReads = 0;
      let chunkReads = 0;
      Object.defineProperty(inner, 'modules', {
        configurable: true,
        get() {
          return nativeModules;
        },
      });
      Object.defineProperty(nativeModules, 'values', {
        configurable: true,
        value() {
          moduleReads++;
          return getModules.call(nativeModules);
        },
      });

      const chunks = compilation.chunks;
      const getChunks = chunks._values;
      Object.defineProperty(chunks, '_values', {
        configurable: true,
        value() {
          chunkReads++;
          return getChunks.call(chunks);
        },
      });

      const hook = compilation.hooks.optimizeTree;
      // Creating the facade before modules are built must not read the graph.
      const retained = compilation.modules;
      if (name === 'empty-sync') {
        hook.tap('LazyCollections', () => {});
      } else if (name === 'empty-promise') {
        hook.tapPromise('LazyCollections', async () => {
          await Promise.resolve();
        });
      } else if (name === 'empty-async') {
        hook.tapAsync('LazyCollections', (_chunks, _modules, callback) => {
          callback();
        });
      } else if (name === 'chunks-only') {
        hook.tap('LazyCollections', (chunks) => {
          assert([...chunks].length > 0);
          assert.equal(moduleReads, 0);
          assert.equal(chunkReads, 1);
        });
      } else if (name === 'size-only') {
        hook.tap('LazyCollections', (_chunks, modules) => {
          assert.equal(modules, retained);
          assert(modules.size >= 2);
          assert.equal(moduleReads, 0);
        });
      } else if (name === 'has-only') {
        hook.tap('LazyCollections', (_chunks, modules) => {
          const chunk = compilation.entrypoints
            .get('main')
            .getEntrypointChunk();
          const [entry] = compilation.chunkGraph
            .getChunkEntryModulesIterable(chunk);
          assert(entry);
          assert.equal(modules.has(entry), true);
          assert.equal(modules.has({}), false);
          assert.equal(modules.has(null), false);
          assert.equal(
            modules.has({ identifier: () => entry.identifier() }),
            false,
          );
          assert.equal(moduleReads, 0);
        });
      } else if (name === 'read-modules') {
        hook.tapPromise('LazyCollections', async (_chunks, modules) => {
          await Promise.resolve();
          assert.equal(moduleReads, 0);
          assert.equal(Object.prototype.toString.call(modules), '[object Set]');
          assert.equal(moduleReads, 0);

          const receiver = {};
          const values = [];
          modules.forEach(function (value, key, set) {
            assert.equal(this, receiver);
            assert.equal(value, key);
            assert.equal(set, modules);
            values.push(value);
          }, receiver);
          assert(values.length >= 2);
          const expected = new Set(values);
          assert.equal(modules.size, expected.size);
          assert.equal(modules.has(values[0]), true);
          assert.equal(modules.has({}), false);
          assert.equal(moduleReads, 1);
          assert.deepEqual([...modules], values);
          assert.deepEqual([...modules.keys()], values);
          assert.deepEqual([...modules.values()], values);
          assert.deepEqual([...modules.entries()], [...expected.entries()]);
          assert.equal(
            util.inspect(modules, { depth: 0 }),
            util.inspect(expected, { depth: 0 }),
          );

          const other = new Set(values.slice(0, 1));
          for (const method of [
            'union',
            'intersection',
            'difference',
            'symmetricDifference',
            'isSubsetOf',
            'isSupersetOf',
            'isDisjointFrom',
          ]) {
            if (typeof Set.prototype[method] === 'function') {
              assert.deepEqual(modules[method](other), expected[method](other));
            }
          }
          // Accessors query the live graph without enumerating the collection.
          const reads = moduleReads;
          const fresh = compilation.modules;
          assert.equal(fresh, modules);
          assert.equal(fresh.size, expected.size);
          assert.equal(moduleReads, reads);
        });
      }

      compilation.hooks.afterSeal.tap('LazyCollections', () => {
        if (name === 'read-modules') {
          assert(moduleReads > 0);
        } else {
          assert.equal(moduleReads, 0);
        }
        assert.equal(chunkReads, name === 'chunks-only' ? 1 : 0);
        delete inner.modules;
        delete nativeModules.values;
        delete chunks._values;
      });
    });
  },
  async build(_context, compiler) {
    // Reusing the compiler also exercises fresh wrappers on the next build.
    for (let i = 0; i < 2; i++) {
      const stats = await runCompiler(compiler);
      assert.equal(
        stats.hasErrors(),
        false,
        stats.toString({ all: false, errors: true }),
      );
    }
  },
}));

cases.push({
  name: 'compiler-identity',
  description: 'checks module ownership and rejects stale compilation collections',
  async run() {
    const options = {
      mode: 'development',
      context: import.meta.dirname,
      entry: './entry.js',
      output: { path: '/dist' },
      optimization: { minimize: false },
    };
    const first = rspack(options);
    const second = rspack(options);
    first.outputFileSystem = createFsFromVolume(new Volume());
    second.outputFileSystem = createFsFromVolume(new Volume());
    let foreignModule;
    const collections = [];
    first.hooks.thisCompilation.tap('ModuleOwnership', (compilation) => {
      compilation.hooks.optimizeTree.tap('ModuleOwnership', (_chunks, modules) => {
        foreignModule = [...modules].find((module) =>
          module.resource?.endsWith('entry.js'),
        );
        assert(foreignModule);
      });
    });
    second.hooks.thisCompilation.tap('ModuleOwnership', (compilation) => {
      const modules = compilation.modules;
      collections.push(modules);
      compilation.hooks.optimizeTree.tap('ModuleOwnership', () => {
        const entry = [...modules].find((module) =>
          module.resource?.endsWith('entry.js'),
        );
        assert(entry);
        assert.equal(entry.identifier(), foreignModule.identifier());
        assert.equal(modules.has(entry), true);
        assert.equal(modules.has(foreignModule), false);
      });
    });
    try {
      assert.equal((await runCompiler(first)).hasErrors(), false);
      assert.equal((await runCompiler(second)).hasErrors(), false);
      assert.equal((await runCompiler(second)).hasErrors(), false);
      assert.throws(() => collections[0].size, /Unable to access compilation/);
      assert(collections[1].size > 0);
    } finally {
      await closeCompiler(first);
      await closeCompiler(second);
    }
  },
});

export default cases;
