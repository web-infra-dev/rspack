const {
  experiments: { RslibPlugin },
} = require('@rspack/core');

module.exports = {
  externalsType: 'modern-module',
  externals: {
    externals: 'externals',
  },
  plugins: [new RslibPlugin()],
};
