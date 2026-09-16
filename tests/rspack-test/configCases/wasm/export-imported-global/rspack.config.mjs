/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index',
  module: {
    rules: [
      {
        test: /\.wat$/,
        loader: 'wast-loader',
        type: 'webassembly/async',
      },
    ],
  },
  experiments: {
    asyncWebAssembly: true,
  },
};
