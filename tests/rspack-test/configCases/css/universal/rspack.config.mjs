/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    module: true,
  },
  target: ['web', 'node'],
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
};
