/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: { type: 'umd' },
  },
  externals: {
    external0: 'external0',
    external1: "var 'abc'",
  },
  node: {
    __dirname: false,
    __filename: false,
  },
};
