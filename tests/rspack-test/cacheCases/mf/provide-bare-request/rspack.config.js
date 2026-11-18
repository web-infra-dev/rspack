const { ProvideSharedPlugin } = require('@rspack/core').sharing;

module.exports = {
  mode: 'development',
  incremental: false,
  cache: { type: 'persistent' },
  plugins: [
    new ProvideSharedPlugin({
      provides: {
        'shared-package': { shareKey: 'exact' },
        'shared-package/': { shareKey: 'prefix/' },
        './included.js': { shareKey: 'included', version: '1.0.0' },
      },
    }),
  ],
};
