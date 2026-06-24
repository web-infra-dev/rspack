/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
  },
  mode: 'development',
  output: {
    pathinfo: false,
  },
  optimization: {
    inlineExports: true,
  },
};
