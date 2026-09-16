/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'node',
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  output: {
    module: true,
    chunkFormat: 'module',
  },
};
