import assert from 'node:assert/strict';
import { writeHeapSnapshot } from 'node:v8';
import { rspack } from '@rspack/core';
const compiler = rspack({
  mode: 'development',
  context: process.cwd(),
  entry: './index.js',
  output: { path: process.env.RSPACK_ALLOCATIVE_DIR },
});
globalThis.retainedStats = [];
globalThis.retainStats = true;
compiler.hooks.afterCompile.tap('AllocativeObjects', compilation => {
  const assets = compilation.getAssets();
  const asset = assets[0];
  assert.equal(Object.getPrototypeOf(asset), Object.prototype);
  const pluginObject = { pluginPayload: true };
  const marker = '__rspack_internal_allocativeObject';
  assert.equal(Object.getOwnPropertyDescriptor(asset, marker)?.enumerable, false);
  assert(!Object.hasOwn(pluginObject, marker));
  asset.rspackAllocativeTestSharedPayload = pluginObject;
  globalThis.allocativeObjects = {
    asset, assets, pluginObject, module: [...compilation.modules][0],
    stats: globalThis.retainStats ? compilation.getStats() : undefined,
    moduleGraph: compilation.moduleGraph,
  };
  if (globalThis.allocativeObjects.stats) {
    globalThis.retainedStats.push(globalThis.allocativeObjects.stats);
    globalThis.statsOwner = compilation;
    writeHeapSnapshot(process.env.ALLOCATIVE_TEST_SNAPSHOT);
  }
});
try {
  await new Promise((resolve, reject) => compiler.run((error, stats) => {
    if (error) reject(error);
    else if (stats.hasErrors()) reject(new Error(stats.toString()));
    else resolve();
  }));
  if (process.env.ALLOCATIVE) {
    await new Promise((resolve, reject) => compiler.__internal__rebuild(
      new Set(), new Set(), error => error ? reject(error) : resolve(),
    ));
    globalThis.retainStats = false;
    globalThis.retainedStats = [];
    globalThis.allocativeObjects.stats = undefined;
    await new Promise(resolve => setImmediate(resolve));
    await new Promise((resolve, reject) => compiler.__internal__rebuild(
      new Set(), new Set(), error => error ? reject(error) : resolve(),
    ));
  }
} finally {
  await new Promise((resolve, reject) => compiler.close(error => error ? reject(error) : resolve()));
}
if (process.env.ALLOCATIVE) {
  globalThis.statsOwner = undefined;
  globalThis.allocativeObjects = undefined;
  await new Promise(resolve => setImmediate(resolve));
  globalThis.gc();
  const next = rspack({
    mode: 'development', context: process.cwd(), entry: './index.js',
    output: { path: process.env.RSPACK_ALLOCATIVE_DIR },
  });
  try {
    await new Promise((resolve, reject) => next.run(error => error ? reject(error) : resolve()));
  } finally {
    await new Promise((resolve, reject) => next.close(error => error ? reject(error) : resolve()));
  }
}
