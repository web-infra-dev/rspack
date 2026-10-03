const { ModuleFederationPlugin } = require('@rspack/core').container;
const path = require('path');

module.exports = {
  target: 'async-node',
  experiments: { layers: true },
  resolve: { alias: { 'second-alias': path.resolve(__dirname, 'second.js') } },
  module: {
    rules: [
      { test: /second\.js$/, layer: 'server' },
      { test: /third\.js$/, layer: 'client' },
      { test: /query\.js$/, use: require.resolve('./query-loader.js') },
    ],
  },
  optimization: { concatenateModules: false },
  plugins: [
    new ModuleFederationPlugin({
      name: 'manifest_import_usage',
      filename: 'container.js',
      manifest: true,
      exposes: {
        './multi': { import: ['./plain.js', 'second-alias'] },
        './other': './third.js',
        './plain': './plain.js',
        './direct': 'shared',
        './scope-scalar': 'scope-scalar',
        './scope-ordered': 'scope-ordered',
        './query-first': './query.js?first',
        './query-second': './query.js?second',
        './consume-only': 'consume-only',
        './direct-multi': { import: ['./plain.js', 'shared'] },
        './direct-layered': { import: 'shared-layered', layer: 'entry-layer' },
      },
      shared: {
        shared: { requiredVersion: false },
        'scope-scalar': {
          shareKey: 'scope-collision',
          shareScope: 'a|b',
          import: false,
          requiredVersion: false,
        },
        'scope-ordered': {
          shareKey: 'scope-collision',
          shareScope: ['a', 'b'],
          import: false,
          requiredVersion: false,
        },
        'query-first': { import: false, requiredVersion: false },
        'query-second': { import: false, requiredVersion: false },
        'consume-only': { import: false, requiredVersion: false },
        'shared-layered': {
          import: 'shared',
          shareScope: 'custom',
          layer: 'shared-layer',
          requiredVersion: false,
        },
      },
    }),
  ],
};
