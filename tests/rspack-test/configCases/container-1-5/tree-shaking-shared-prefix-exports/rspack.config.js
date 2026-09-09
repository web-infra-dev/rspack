const { ModuleFederationPlugin } = require('@rspack/core').container;

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  target: 'async-node',
  experiments: { layers: true },
  module: { rules: [{ test: /server\.js$/, layer: 'server' }] },
  plugins: [
    new ModuleFederationPlugin({
      name: 'tree_shaking_shared_prefix_exports',
      manifest: true,
      shared: {
        'exact-directory': {
          shareKey: 'directory',
          import: './directory/',
          version: '1.0.0',
          requiredVersion: false,
          treeShaking: { mode: 'runtime-infer' },
        },
        'exact-file': {
          shareKey: 'directorysub.js',
          import: './directory/sub.js',
          version: '1.0.0',
          requiredVersion: false,
          treeShaking: { mode: 'runtime-infer' },
        },
        'prefix/': {
          shareKey: 'custom-',
          requiredVersion: false,
          treeShaking: { mode: 'runtime-infer', usedExports: ['manual'] },
        },
        'prefix/deep/': {
          shareKey: 'deep-',
          requiredVersion: false,
          treeShaking: { mode: 'runtime-infer' },
        },
        'prefix/exact': {
          shareKey: 'exact',
          requiredVersion: false,
          treeShaking: { mode: 'runtime-infer' },
        },
        'prefix/disabled': {
          shareKey: 'custom-disabled',
          requiredVersion: false,
        },
        'prefix/disabled/': {
          shareKey: 'custom-disabled/',
          requiredVersion: false,
        },
        'overlap/': {
          shareKey: 'manual-long-',
          requiredVersion: false,
          treeShaking: { mode: 'runtime-infer', usedExports: ['manualWrong'] },
        },
        'overlap/deep/': {
          shareKey: 'manual-',
          requiredVersion: false,
          treeShaking: { mode: 'runtime-infer', usedExports: ['manualRight'] },
        },
        server: {
          request: 'prefix/',
          shareKey: 'server-',
          shareScope: ['server', 'default'],
          issuerLayer: 'server',
          layer: 'server',
          requiredVersion: false,
          treeShaking: { mode: 'runtime-infer', usedExports: ['manualServer'] },
        },
      },
    }),
  ],
};
