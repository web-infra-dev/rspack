const { rspack } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
  plugins: [
    new rspack.ProvidePlugin({
      PROVIDED_VALUE: './provided.cjs',
    }),
  ],
};
