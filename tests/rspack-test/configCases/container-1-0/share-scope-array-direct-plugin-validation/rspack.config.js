const { ConsumeSharedPlugin } = require('@rspack/core').sharing;

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'development',
  entry: './index.js',
  plugins: [
    new ConsumeSharedPlugin({
      consumes: ['react'],
      shareScope: ['default', 'ssr'],
    }),
  ],
};
