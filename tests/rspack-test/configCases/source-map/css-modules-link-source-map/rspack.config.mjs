/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'node',
  mode: 'development',
  devtool: 'source-map',
  externals: ['source-map'],
  externalsType: 'commonjs',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
};
