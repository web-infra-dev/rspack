const { ProvidePlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  optimization: {
    moduleIds: 'named',
    inlineExports: true,
  },
  plugins: [
    new ProvidePlugin({
      providedA: ['./constants.js', 'a'],
      providedDefault: ['./constants.js', 'default'],
    }),
  ],
};
