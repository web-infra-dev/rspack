const assert = require('node:assert/strict');
const path = require('node:path');
const { rspack } = require('@rspack/core');
const seen = [];
let enabled;
const retained = [];
const check = (object) => {
  assert(object);
  const pointer = Object.getOwnPropertyDescriptor(object, '__rspackRustPtr');
  if (enabled === undefined) enabled = Boolean(pointer);
  else assert.equal(Boolean(pointer), enabled, object.constructor.name);
  if (!pointer) {
    assert.equal(Object.hasOwn(object, '__rspackRustType'), false);
    return;
  }
  assert.equal(pointer.enumerable, false);
  assert.equal(pointer.writable, false);
  assert.match(pointer.value, /^0x[0-9a-f]+$/);
  assert.equal(typeof object.__rspackRustType, 'string');
  seen.push({ type: object.constructor.name, pointer: pointer.value });
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
            check(factory);
            check(factory.get('normal', {}));
            for (const chunk of compilation.chunks) check(chunk);
          });
        });
      },
    },
  ],
});
compiler.run((error, stats) => {
  if (error) {
    console.error(error);
    compiler.close(() => {
      process.exitCode = 1;
    });
    return;
  }
  assert.equal(stats.hasErrors(), false, stats.toString());
  console.log(JSON.stringify({ enabled, seen }));
  compiler.close((error) => {
    if (error) throw error;
  });
});
