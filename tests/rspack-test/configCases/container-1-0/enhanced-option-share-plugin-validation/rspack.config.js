const { SharePlugin } = require('@rspack/core').sharing;

module.exports = {
  entry: './index.js',
  plugins: [
    new SharePlugin({
      enhanced: false,
      shared: {
        react: {
          layer: 'server',
        },
      },
    }),
  ],
};
