const assert = require('node:assert/strict');
const { createRequire } = require('node:module');
const path = require('node:path');
const binding = createRequire(require.resolve('@rspack/core/package.json'))(
  '@rspack/binding',
);
assert.equal(
  Object.hasOwn(binding, 'registerAllocativeSnapshotCallback'),
  false,
  'the native binding must not export a snapshot registration function',
);
const hookName = '__rspack_internal_runJsSnapshot';
let snapshotCalls = 0;
const { rspack } = require('@rspack/core');
const seen = [];
let enabled;
const retained = [];
const check = (object, source) => {
  assert(object);
  const label = source ?? object.constructor.name;
  const pointer = Object.getOwnPropertyDescriptor(object, '__rspackRustPtr');
  if (enabled === undefined) enabled = Boolean(pointer);
  else assert.equal(Boolean(pointer), enabled, label);
  if (!pointer) {
    assert.equal(Object.hasOwn(object, '__rspackRustType'), false);
    return;
  }
  assert.equal(pointer.enumerable, false);
  assert.equal(pointer.writable, false);
  assert.match(pointer.value, /^0x[0-9a-f]+$/);
  assert.equal(typeof object.__rspackRustType, 'string');
  seen.push({ label, pointer: pointer.value });
  retained.push(object);
};
const compiler = rspack({
  mode: 'development',
  context: process.env.ALLOCATIVE_TEST_CONTEXT,
  entry: './index.js',
  output: { path: path.join(process.env.ALLOCATIVE_TEST_CONTEXT, 'dist') },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.thisCompilation.tap('SnapshotFixture', (compilation) => {
          check(compilation.__internal_getInner());
          if (enabled) {
            const runSnapshot = globalThis[hookName];
            Object.defineProperty(globalThis, hookName, {
              value(snapshotPath) {
                snapshotCalls++;
                runSnapshot(snapshotPath);
              },
            });
          }

          compilation.hooks.finishModules.tap('SnapshotFixture', (modules) => {
            for (const module of modules) {
              // Module is already the native instance in the public API.
              check(module);
              for (const dependency of module.dependencies ?? [])
                check(dependency);
            }
          });
          compilation.hooks.afterSeal.tap('SnapshotFixture', () => {
            const raw = compilation.__internal_getInner();
            check(raw.moduleGraph);
            check(raw.chunkGraph);
            check(raw.getStats());
            const factory = compiler.resolverFactory.constructor.__to_binding(
              compiler.resolverFactory,
            );
            check(factory, 'resolver factory created before the compiler');

            const constructed = new binding.JsResolverFactory(false, {}, {});
            assert(constructed instanceof binding.JsResolverFactory);
            check(constructed, 'JavaScript constructor');

            const returned = constructed.get('normal', {});
            assert(returned instanceof binding.JsResolver);
            check(returned, 'Rust return value');

            const fromFactory = binding.JsResolverFactory.__testCreate(
              false,
              {},
              {},
            );
            assert(fromFactory instanceof binding.JsResolverFactory);
            check(fromFactory, 'N-API factory');

            for (const chunk of compilation.chunks) check(chunk);
          });
        });
      },
    },
  ],
});
const hook = Object.getOwnPropertyDescriptor(globalThis, hookName);
if (process.env.RSPACK_ALLOCATIVE_DIR === undefined) {
  assert.equal(hook, undefined);
} else {
  assert.equal(typeof hook?.value, 'function');
  assert.equal(hook.enumerable, false);
  assert.equal(hook.writable, false);
}

compiler.run((error, stats) => {
  if (error) {
    console.error(error);
    compiler.close(() => {
      process.exitCode = 1;
    });
    return;
  }
  assert.equal(stats.hasErrors(), false, stats.toString());
  assert.equal(snapshotCalls, enabled ? 1 : 0);
  console.log(JSON.stringify({ enabled, seen }));
  compiler.close((error) => {
    if (error) throw error;
  });
});
