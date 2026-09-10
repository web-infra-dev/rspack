const { ModuleFederationPlugin } = require('@rspack/core').container;

const explicit = {
  'alias-a': {
    import: 'pkg-a',
    version: '2.0.0',
    shareKey: 'same-key',
    requiredVersion: '^2.0.0',
    singleton: true,
    strictVersion: true,
    treeShaking: { mode: 'runtime-infer' },
  },
};

/** @type {import('@rspack/core').Configuration[]} */
module.exports = [false, true].map((explicitFirst) => ({
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
      name: `tree_shaking_shared_request_origins_${explicitFirst}`,
      treeShakingSharedDir: `independent-packages-${explicitFirst}`,
      library: { type: 'commonjs-module' },
      shared: [
        ...(explicitFirst ? [explicit, explicit] : []),
        {
          'relative-share': {
            import: './shared',
            shareKey: 'relative',
            requiredVersion: false,
            treeShaking: { mode: 'runtime-infer' },
          },
          'alias-a': {
            import: 'pkg-a',
            shareKey: 'same-key',
            requiredVersion: false,
            treeShaking: { mode: 'runtime-infer' },
          },
          'alias-a-unversioned': {
            import: 'pkg-a',
            version: false,
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
            import: 'pkg-a?copy#fragment',
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
        ...(explicitFirst ? [] : [explicit, explicit]),
      ],
    }),
  ],
}));
