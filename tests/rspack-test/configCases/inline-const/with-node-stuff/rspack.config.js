/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
  },
  target: 'web',
  mode: 'production',
  node: {
    global: true,
    __filename: false,
  },
  optimization: {
    minimize: false,
    inlineExports: true,
  },
};
