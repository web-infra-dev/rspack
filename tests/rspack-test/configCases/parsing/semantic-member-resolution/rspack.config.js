const { rspack } = require('@rspack/core');

module.exports = {
  target: 'node',
  module: { parser: { javascript: { requireAlias: true } } },
  plugins: [
    new rspack.DefinePlugin({
      'DEFINED.branch.value': JSON.stringify('defined'),
    }),
  ],
};
