/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    moduleIds: 'named',
  },
  output: {
    webassemblyModuleFilename: '[id].[hash].wasm',
  },
  experiments: {
    asyncWebAssembly: true,
  },
};
