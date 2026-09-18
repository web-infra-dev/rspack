/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  output: {
    filename: '[name].js',
  },
  externals: {
    fs: 'commonjs2 fs',
  },
  node: {
    __filename: false,
    __dirname: false,
  },
};
