const rspack = require('@rspack/core');

module.exports = {
  mode: 'production',
  plugins: [
    new rspack.DefinePlugin({
      FALSY: JSON.stringify(false),
    }),
  ],
};
