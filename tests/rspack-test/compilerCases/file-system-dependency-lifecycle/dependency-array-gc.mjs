import assert from 'node:assert/strict';
import path from 'node:path';
import { setImmediate } from 'node:timers/promises';
import { rspack } from '@rspack/core';
import {
  runCompiler,
  closeCompiler,
} from '@rspack/test-tools/helper/lifecycle';

async function expectCollected(ref, message) {
  for (let attempt = 0; attempt < 100; attempt++) {
    await setImmediate();
    global.gc();
    if (!ref.deref()) return;
  }
  assert.fail(message);
}

export default async function run() {
  const compiler = rspack({ mode: 'development', entry: {}, cache: false });
  let generation = 0;
  let retainedCompilation;
  let dependencies;
  let arrayRef;
  let compilationRef;
  const firstPath = path.resolve('dependency-array-first');
  const secondPath = path.resolve('dependency-array-second');
  compiler.hooks.thisCompilation.tap(
    'FileSystemDependencyArrayGc',
    (compilation) => {
      const dependency = generation++ === 0 ? firstPath : secondPath;
      compilation.fileDependencies.add(dependency);
      assert(compilation.fileDependencies.has(dependency));
      if (generation === 1) {
        retainedCompilation = compilation;
        const inner = compilation.__internal_getInner();
        dependencies = inner.fileDependencies;
        const array = dependencies.values();
        assert(
          Object.getOwnPropertySymbols(dependencies).some(
            (symbol) => dependencies[symbol] === array,
          ),
        );
        arrayRef = new WeakRef(array);
        compilationRef = new WeakRef(inner);
      }
    },
  );
  try {
    await runCompiler(compiler);
    assert(arrayRef.deref().includes(firstPath));
    await runCompiler(compiler);
    await setImmediate();
    global.gc();
    assert(arrayRef.deref(), 'the dependency wrapper owns its result array');
    assert(retainedCompilation);
    retainedCompilation = undefined;
    await expectCollected(compilationRef, 'the compilation should be collected');
    assert(
      arrayRef.deref(),
      'the array survives compilation GC with its wrapper',
    );
    let values = dependencies.values();
    assert.strictEqual(values, arrayRef.deref());
    assert(
      values.includes(secondPath),
      'retained wrappers read the current compilation',
    );
    assert(!values.includes(firstPath));
    values = undefined;
    dependencies = undefined;
    await expectCollected(arrayRef, 'the array is collected with its wrapper');
  } finally {
    await closeCompiler(compiler);
  }
}
