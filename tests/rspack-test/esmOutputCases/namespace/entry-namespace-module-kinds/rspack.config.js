const { CopyRspackPlugin } = require('@rspack/core');

module.exports = {
  externals: {
    'live-external': 'module ./live-external.mjs',
  },
  plugins: [
    new CopyRspackPlugin({
      patterns: [{ from: 'live-external.mjs', to: 'live-external.mjs' }],
    }),
  ],
};
