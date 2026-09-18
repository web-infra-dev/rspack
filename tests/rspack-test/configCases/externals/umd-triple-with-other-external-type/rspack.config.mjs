/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  externals: {
    pkg: {
      root: 'pkg',
      commonjs: 'pkg',
      commonjs2: 'pkg',
      amd: 'pkg',
    },
  },
  optimization: {
    concatenateModules: true,
  },
};
