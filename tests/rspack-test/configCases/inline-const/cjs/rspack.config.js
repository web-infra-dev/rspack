/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
  },
  entry: './index.cjs',
  optimization: {
    moduleIds: 'named',
    inlineExports: true,
  },
};
