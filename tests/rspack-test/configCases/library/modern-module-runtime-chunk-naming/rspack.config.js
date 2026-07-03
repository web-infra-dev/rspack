const rspack = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
const basic = {
  output: {
    filename: '[name].js',
    module: true,
    library: {
      type: 'modern-module',
    },
  },
  plugins: [new rspack.experiments.RslibPlugin()],
  optimization: {
    concatenateModules: true,
    avoidEntryIife: true,
    minimize: false,
    // Force the esm-library's own runtime split (the rslib scenario) instead of
    // the test harness default `runtimeChunk: { name: 'runtime~<index>' }`.
    runtimeChunk: false,
  },
};

// Two libs share one output directory. Each pulls in a CommonJS dep (cannot be
// scope-hoisted, so the runtime registers it via `__webpack_require__.add`) and
// a dynamic import (so the runtime is split into a separate *initial* chunk).
// That runtime chunk must be named after its entry, otherwise it falls back to
// a numeric id and the two libs collide on the same `<id>.js`.
module.exports = [
  { entry: { a: './a.js' }, ...basic },
  { entry: { b: './b.js' }, ...basic },
  {
    entry: { index: './index.js' },
    output: { module: true, filename: 'index.mjs' },
  },
];
