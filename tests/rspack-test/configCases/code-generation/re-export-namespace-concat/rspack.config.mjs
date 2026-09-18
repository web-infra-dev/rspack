/** @type {import("@rspack/core").Configuration} */
export default {
  node: {
    __dirname: false,
    __filename: false,
  },
  mode: 'production',
  optimization: {
    mangleExports: 'size',
    inlineExports: false,
  },
};
