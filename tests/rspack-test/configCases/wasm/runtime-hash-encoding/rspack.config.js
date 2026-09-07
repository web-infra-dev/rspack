/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node',
  module: {
    rules: [
      {
        test: /\.wat$/,
        loader: 'wast-loader',
        type: 'webassembly/async',
      },
    ],
  },
  output: {
    webassemblyModuleFilename: '[id].[hash:base64:8].wasm',
  },
  experiments: {
    asyncWebAssembly: true,
  },
};
