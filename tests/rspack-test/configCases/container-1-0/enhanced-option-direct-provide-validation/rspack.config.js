const { ProvideSharedPlugin } = require('@rspack/core').sharing;

module.exports = {
  entry: './index.js',
  plugins: [
    new ProvideSharedPlugin({
      provides: {
        react: {
          shareKey: 'react',
          layer: 'server',
        },
      },
    }),
  ],
};
