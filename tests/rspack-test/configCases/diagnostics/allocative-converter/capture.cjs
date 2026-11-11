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
let afterCompile = false;
let done = false;
const { rspack } = require('@rspack/core');
if (process.env.ALLOCATIVE_TEST_CONTENT) {
  const content = process.env.ALLOCATIVE_TEST_CONTENT;
  globalThis[content] = { [content]: content };
}
const seen = [];
let enabled;
let releasedAfterSnapshot;
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
// A binding can be used without installing the optional JS snapshot hook.
const env = process.env;
const profileDirectory = env.RSPACK_ALLOCATIVE_DIR;
if (env.ALLOCATIVE_TEST_RUST_ONLY) delete env.RSPACK_ALLOCATIVE_DIR;
let directoryReads = 0;
process.env = new Proxy(env, {
  get(target, key, receiver) {
    if (key === 'RSPACK_ALLOCATIVE_DIR') directoryReads++;
    return Reflect.get(target, key, receiver);
  },
});
const compiler = rspack({
  mode: 'development',
  context: process.env.ALLOCATIVE_TEST_CONTEXT,
  entry: './index.js',
  output: { path: path.join(process.env.ALLOCATIVE_TEST_CONTEXT, 'dist') },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.afterCompile.tap('SnapshotPhase', () => {
          afterCompile = true;
        });
        compiler.hooks.done.tap('SnapshotPhase', () => {
          done = true;
        });
        compiler.hooks.thisCompilation.tap('SnapshotFixture', (compilation) => {
          check(compilation.__internal_getInner());
          if (enabled) {
            const runSnapshot = globalThis[hookName];
            Object.defineProperty(globalThis, hookName, {
              value(snapshotPath) {
                snapshotCalls++;
                assert(afterCompile, 'capture must follow core compilation');
                assert(
                  !done,
                  'capture must finish before the JS done callback',
                );
                const stringify = JSON.stringify;
                let pending;
                try {
                  // Model the V8 limit without putting a 512 MiB snapshot in CI.
                  JSON.stringify = (value, ...args) => {
                    assert(
                      !value || !Object.hasOwn(value, 'rspack_binding_nodes'),
                      'redacted snapshots must be serialized in chunks',
                    );
                    assert(
                      !Array.isArray(value) || value.length <= 16_384,
                      'large snapshot arrays must be serialized in chunks',
                    );
                    return stringify(value, ...args);
                  };
                  pending = runSnapshot(snapshotPath);
                } finally {
                  JSON.stringify = stringify;
                }
                assert.equal(
                  pending?.then,
                  undefined,
                  'capture must stay in one JS-thread callback',
                );
                if (releasedAfterSnapshot) {
                  queueMicrotask(() => {
                    releasedAfterSnapshot = undefined;
                    globalThis.gc?.();
                  });
                }
                return pending;
              },
            });
          }

          compilation.hooks.finishModules.tap('SnapshotFixture', (modules) => {
            for (const module of modules) {
              // The JavaScript Module wraps a Rust ModuleHandle.
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

            if (
              enabled &&
              process.env.RSPACK_ALLOCATIVE_INCLUDE_CONTENT !== '1'
            ) {
              // This wrapper is live in the heap graph but can die in a microtask after capture returns.
              releasedAfterSnapshot = new binding.JsResolverFactory(
                false,
                {},
                {},
              );
            }

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
if (env.ALLOCATIVE_TEST_RUST_ONLY) env.RSPACK_ALLOCATIVE_DIR = profileDirectory;
const hook = Object.getOwnPropertyDescriptor(globalThis, hookName);
if (env.RSPACK_ALLOCATIVE_DIR === undefined || env.ALLOCATIVE_TEST_RUST_ONLY) {
  assert.equal(hook, undefined);
} else {
  assert.equal(typeof hook?.value, 'function');
  assert.equal(hook.enumerable, false);
  assert.equal(hook.writable, false);
}

compiler.run((error, stats) => {
  process.env = env;
  if (error) {
    console.error(error);
    compiler.close(() => {
      process.exitCode = 1;
    });
    return;
  }
  assert.equal(directoryReads, 1, 'JS must read the snapshot directory once');
  assert.equal(stats.hasErrors(), false, stats.toString());
  assert.equal(snapshotCalls, enabled ? 1 : 0);
  console.log(JSON.stringify({ enabled, seen }));
  compiler.close((error) => {
    if (error) throw error;
  });
});
