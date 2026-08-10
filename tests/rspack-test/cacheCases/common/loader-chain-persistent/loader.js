const loaderRuns = Symbol.for('rspack.test.loaderChainPersistentRuns');
globalThis[loaderRuns] = globalThis[loaderRuns] || 0;

module.exports = function (source) {
  globalThis[loaderRuns]++;
  return source.replace('__RUNS__', globalThis[loaderRuns]);
};
---
const loaderRuns = Symbol.for('rspack.test.loaderChainPersistentRuns');
globalThis[loaderRuns] = globalThis[loaderRuns] || 0;

module.exports = function (source) {
  globalThis[loaderRuns]++;
  return source.replace('__RUNS__', globalThis[loaderRuns]);
};
---
const loaderRuns = Symbol.for('rspack.test.loaderChainPersistentRuns');
globalThis[loaderRuns] = globalThis[loaderRuns] || 0;

// Changing the loader implementation must invalidate its cached chain.
module.exports = function (source) {
  globalThis[loaderRuns]++;
  return source.replace('__RUNS__', globalThis[loaderRuns]);
};
