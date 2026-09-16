/** @type {import("@rspack/core").Configuration} */
export default {
  externals: {
    fs: 'node-commonjs fs',
  },
  optimization: {
    inlineExports: true,
  },
};
