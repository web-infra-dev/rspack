const { container } = require('@rspack/core');

const { ModuleFederationPlugin } = container;

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  target: 'async-node',
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  optimization: {
    minimize: false,
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'shared_identities',
      enhanced: true,
      manifest: true,
      shareScope: ['root', 'default'],
      shared: {
        'variant-a': {
          request: 'variant-a',
          shareKey: 'shared-variant',
          shareScope: ['scope-a', 'default'],
          layer: 'layer-a',
          requiredVersion: '*',
          treeShaking: { mode: 'runtime-infer' },
        },
        'variant-b': {
          request: 'variant-b',
          shareKey: 'shared-variant',
          shareScope: ['scope-b', 'default'],
          layer: 'layer-b',
          requiredVersion: '*',
          treeShaking: { mode: 'runtime-infer' },
        },
        'legacy-shared': {
          requiredVersion: '*',
          treeShaking: { mode: 'runtime-infer' },
        },
        'default-unlayered': {
          shareKey: 'default-collision',
          shareScope: 'default',
          requiredVersion: '*',
          treeShaking: { mode: 'runtime-infer' },
        },
        'default-layered': {
          shareKey: 'default-collision',
          shareScope: 'default',
          layer: 'layered',
          requiredVersion: '*',
          treeShaking: { mode: 'runtime-infer' },
        },
        'custom-unlayered': {
          shareKey: 'default-collision',
          shareScope: 'custom',
          requiredVersion: '*',
          treeShaking: { mode: 'runtime-infer' },
        },
      },
    }),
  ],
};
