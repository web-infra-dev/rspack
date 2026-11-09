const { ProvideSharedPlugin } = require('@rspack/core').sharing;

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'development',
  entry: './index.js',
  plugins: [
    new ProvideSharedPlugin({
      provides: ['react'],
      shareScope: ['default', 'ssr'],
    }),
  ],
};
