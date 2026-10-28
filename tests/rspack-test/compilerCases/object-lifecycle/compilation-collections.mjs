import assert from 'node:assert/strict';
import { rspack } from '@rspack/core';
import { createFsFromVolume, Volume } from 'memfs';
import {
  closeCompiler,
  createGCTracker,
  runCompiler,
} from '@rspack/test-tools/helper/lifecycle';

export default async function run() {
  const tracker = createGCTracker();
  let retainedCollections;
  let compiler = rspack({
    context: import.meta.dirname,
    mode: 'development',
    entry: './entry.js',
    output: { path: '/', filename: 'bundle.js' },
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());
  compiler.hooks.make.tapPromise('CompilationCollections', async (compilation) => {
    const inner = compilation.__internal_getInner();
    // Drop all strong collection references before awaiting GC, while keeping
    // the compilation alive. Reading a getter must not read the module graph.
    ['modules', 'builtModules', 'chunks'].forEach((name) => {
      const collection = inner[name];
      assert.equal(inner[name], collection);
      const view = compilation[name];
      assert.equal(compilation[name], view);
      tracker.track(view, `${name} first view`);
      tracker.track(collection, `${name} first instance`);
    });
    for (const name of ['modules', 'builtModules', 'chunks']) {
      await tracker.waitForCollection(`${name} first view`);
      await tracker.waitForCollection(`${name} first instance`);
    }
    retainedCollections = ['modules', 'builtModules', 'chunks'].map((name) => {
      const collection = inner[name];
      assert.equal(inner[name], collection);
      tracker.track(collection, `${name} recreated instance`);
      return collection;
    });
  });
  compiler.hooks.done.tap('CompilationCollections', ({ compilation }) => {
    const inner = compilation.__internal_getInner();
    ['modules', 'builtModules', 'chunks'].forEach((name, index) => {
      const collection = retainedCollections[index];
      assert.equal(inner[name], collection);
      assert(collection.size() > 0);
      const values = collection.values();
      assert.equal(collection.values(), values);
      tracker.track(values, `${name} values`);
      assert.equal(values.length, collection.size());
      assert(collection.has(values[0]));
    });
  });
  try {
    await runCompiler(compiler);
  } finally {
    await closeCompiler(compiler);
  }
  tracker.track(compiler, 'collection compiler');
  compiler = null;
  await tracker.waitForCollection('collection compiler');
  retainedCollections.forEach((collection) => {
    assert.throws(() => collection.size(), /Unable to access compilation/);
    assert.throws(() => collection.has({}), /Unable to access compilation/);
    assert.throws(() => collection.values(), /Unable to access compilation/);
  });
  retainedCollections = null;
  for (const name of ['modules', 'builtModules', 'chunks']) {
    await tracker.waitForCollection(`${name} recreated instance`);
    await tracker.waitForCollection(`${name} values`);
  }
}
