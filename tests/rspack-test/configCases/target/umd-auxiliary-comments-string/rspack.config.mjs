/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: {
      type: 'umd',
      name: 'NamedLibrary',
      umdNamedDefine: true,
      auxiliaryComment: 'test comment',
    },
  },
  node: {
    __dirname: false,
    __filename: false,
  },
};
