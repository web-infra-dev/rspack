const { DefinePlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  plugins: [
    new DefinePlugin({
      FEATURE_ENABLED: 'false',
    }),
  ],
  optimization: {
    concatenateModules: false,
  },
  output: {
    chunkFilename: '[name].js',
  },
};
