const { ModuleFederationPlugin } = require('@rspack/core').container;

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  target: 'async-node',
  output: {
    publicPath: 'PUBLIC_PATH',
    chunkFilename: '[id].js',
  },
  resolve: {
    alias: {
      'alias-a': require.resolve('./node_modules/pkg-a'),
      'alias-b': require.resolve('./node_modules/pkg-b'),
      'alias-b-alt': require.resolve('./node_modules/pkg-b'),
      'alias-a-copy': require.resolve('./node_modules/pkg-a'),
      'alias-a-query': require.resolve('./node_modules/pkg-a'),
      'pkg-a-copy': require.resolve('./node_modules/pkg-a'),
    },
  },
  module: {
    rules: [
      { resourceQuery: /copy/, loader: require.resolve('./query-loader.js') },
    ],
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'tree_shaking_shared_request_origins',
      library: { type: 'commonjs-module' },
      shared: {
        'alias-a': {
          import: 'pkg-a',
          shareKey: 'same-key',
          requiredVersion: false,
          treeShaking: { mode: 'runtime-infer' },
        },
        'alias-b': {
          import: 'pkg-b',
          version: '2.0.0',
          shareKey: 'same-key',
          requiredVersion: false,
          treeShaking: { mode: 'runtime-infer' },
        },
        'alias-b-alt': {
          import: 'pkg-b',
          version: '3.0.0',
          shareKey: 'same-key',
          requiredVersion: false,
          treeShaking: { mode: 'runtime-infer' },
        },
        'alias-a-copy': {
          import: 'pkg-a-copy',
          shareKey: 'same-key',
          requiredVersion: false,
          treeShaking: { mode: 'runtime-infer' },
        },
        'alias-a-query': {
          import: 'pkg-a?copy',
          shareKey: 'same-key',
          requiredVersion: false,
          treeShaking: { mode: 'runtime-infer' },
        },
        'pkg/': {
          shareKey: 'prefix/',
          requiredVersion: false,
          treeShaking: { mode: 'runtime-infer' },
        },
        'mapped-prefix': {
          request: 'mapped/',
          import: 'pkg-',
          shareKey: 'mapped/',
          requiredVersion: false,
          treeShaking: { mode: 'runtime-infer' },
        },
      },
    }),
  ],
};
