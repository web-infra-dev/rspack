const {
  experiments: { RslibPlugin },
} = require('@rspack/core');

module.exports = {
  externals: {
    fs: 'commonjs fs',
  },
  plugins: [new RslibPlugin()],
};
