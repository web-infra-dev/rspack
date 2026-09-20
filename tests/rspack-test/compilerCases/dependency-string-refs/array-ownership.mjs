import assert from 'node:assert/strict';
import path from 'node:path';
import { setImmediate } from 'node:timers/promises';
import { rspack } from '@rspack/core';
import {
  runCompiler,
  closeCompiler,
} from '@rspack/test-tools/helper/lifecycle';

export default async function run() {
  const compiler = rspack({ mode: 'development', entry: {}, cache: false });
  let generation = 0;
  let retainedCompilation;
  let dependencies;
  let arrayRef;
  let compilationRef;
  let snapshot;
  const firstPath = path.resolve('array-owner-first');
  const secondPath = path.resolve('array-owner-second');
  compiler.hooks.thisCompilation.tap('ArrayOwnership', (compilation) => {
    compilation.fileDependencies.add(
      generation++ === 0 ? firstPath : secondPath,
    );
    assert(
      compilation.fileDependencies.has(
        generation === 1 ? firstPath : secondPath,
      ),
    );
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
      snapshot = array.slice();
    }
  });
  try {
    await runCompiler(compiler);
    await runCompiler(compiler);
    await setImmediate();
    global.gc();
    assert(arrayRef.deref(), 'the dependency wrapper owns its result array');
    assert(retainedCompilation);
    retainedCompilation = undefined;
    for (let attempt = 0; attempt < 100; attempt++) {
      await setImmediate();
      global.gc();
      if (!compilationRef.deref()) break;
    }
    assert.equal(compilationRef.deref(), undefined);
    assert(
      arrayRef.deref(),
      'the array survives compilation GC with its wrapper',
    );
    assert(snapshot.includes(firstPath));
    for (let attempt = 0; attempt < 3; attempt++) {
      await setImmediate();
      global.gc();
      assert.strictEqual(dependencies.values(), arrayRef.deref());
      assert(
        dependencies.values().includes(secondPath),
        'retained wrappers read the current compilation',
      );
      assert(!dependencies.values().includes(firstPath));
    }
    dependencies = undefined;
    for (let attempt = 0; attempt < 100; attempt++) {
      await setImmediate();
      global.gc();
      if (!arrayRef.deref()) break;
    }
    assert.equal(
      arrayRef.deref(),
      undefined,
      'the array is collected with its wrapper',
    );
  } finally {
    await closeCompiler(compiler);
  }
}
