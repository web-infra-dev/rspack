/** @typedef {import("@rspack/core").Compiler} Compiler */

/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    webassemblyModuleFilename: '[id].[hash].wasm',
  },
  experiments: {
    asyncWebAssembly: true,
  },
};
