const { ConsumeSharedPlugin } = require('@rspack/core').sharing;

module.exports = {
  plugins: [
    new ConsumeSharedPlugin({
      consumes: {
        first: {
          shareKey: 'shared',
          import: './fallback.js?a',
          requiredVersion: false,
        },
        second: {
          shareKey: 'shared',
          import: './fallback.js?b',
          requiredVersion: false,
        },
      },
    }),
  ],
};
