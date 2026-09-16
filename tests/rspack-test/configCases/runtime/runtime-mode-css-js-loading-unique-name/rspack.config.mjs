/** @type {import("@rspack/core").Configuration} */
export default {
  experiments: {
    runtimeMode: 'rspack',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  output: {
    uniqueName: 'runtime-review',
  },
  target: 'web',
};
