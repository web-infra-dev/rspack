/** @type {import("@rspack/core").Configuration} */
module.exports = {
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
