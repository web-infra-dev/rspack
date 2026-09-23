const { ConsumeSharedPlugin } = require('@rspack/core').sharing;

module.exports = {
  entry: './index.js',
  plugins: [
    new ConsumeSharedPlugin({
      consumes: {
        react: {
          import: false,
          request: 'react-server',
        },
      },
    }),
  ],
};
