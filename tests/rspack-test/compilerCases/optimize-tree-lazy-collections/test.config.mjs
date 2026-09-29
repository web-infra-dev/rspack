import assert from 'node:assert/strict';
import util from 'node:util';
import { runCompiler } from '@rspack/test-tools/helper/lifecycle';

const scenarios = [
  'no-hook',
  'empty-sync',
  'empty-promise',
  'empty-async',
  'chunks-only',
  'read-modules',
];

export default scenarios.map((name) => ({
  name,
  description: `materializes only the optimizeTree collections used by ${name}`,
  options: () => ({ mode: 'development', entry: './entry.js' }),
  compiler(_context, compiler) {
    compiler.hooks.thisCompilation.tap('LazyCollections', (compilation) => {
      const inner = compilation.__internal_getInner();
      const descriptor = Object.getOwnPropertyDescriptor(
        Object.getPrototypeOf(inner),
        'modules',
      );
      assert(descriptor?.get);
      let moduleReads = 0;
      let chunkReads = 0;
      Object.defineProperty(inner, 'modules', {
        configurable: true,
        get() {
          moduleReads++;
          return descriptor.get.call(inner);
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
          assert.equal(moduleReads, 1);

          // A new getter access must not reuse a snapshot from an earlier phase.
          const fresh = compilation.modules;
          assert.notEqual(fresh, modules);
          assert.equal(moduleReads, 1);
          assert.equal(fresh.size, expected.size);
          assert.equal(moduleReads, 2);
        });
      }

      compilation.hooks.afterSeal.tap('LazyCollections', () => {
        assert.equal(moduleReads, name === 'read-modules' ? 2 : 0);
        assert.equal(chunkReads, name === 'chunks-only' ? 1 : 0);
        delete inner.modules;
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
