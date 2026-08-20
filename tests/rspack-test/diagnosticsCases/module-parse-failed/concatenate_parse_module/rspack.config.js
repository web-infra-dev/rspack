const { rspack } = require('@rspack/core');

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  experiments: {
    // This case verifies the diagnostic produced by the legacy concatenation reparse.
    fasterModuleConcatenation: false,
  },
  plugins: [
    new rspack.DefinePlugin({
      DEFINE_VAR: '1 2 3',
    }),
  ],
  optimization: {
    concatenateModules: true,
  },
};
