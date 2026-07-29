const { ModuleFederationPlugin } = require('@rspack/core').container;

const createConfig = (fileName, disableAssetsAnalyze) => ({
  entry: './entry.js',
  target: 'async-node',
  experiments: {
    layers: true,
  },
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
  },
  output: {
    filename: `${fileName}/[name].js`,
    chunkFilename: `${fileName}/[id].js`,
    uniqueName: `manifest-shared-identity-${fileName}`,
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      manifest: {
        fileName: `${fileName}.json`,
        disableAssetsAnalyze,
      },
      exposes: {
        './first': {
          import: './exposed.js',
          layer: 'first-layer',
        },
        './second': {
          import: './exposed.js',
          layer: 'second-layer',
        },
      },
      shared: {
        legacy: {
          version: false,
          requiredVersion: false,
          shareScope: 'custom',
        },
        'legacy-a': {
          request: 'legacy-a',
          shareKey: 'collision',
          version: '1.0.0',
          requiredVersion: false,
          shareScope: 'scope-a',
        },
        'legacy-b': {
          request: 'legacy-b',
          shareKey: 'collision',
          version: '1.0.0',
          requiredVersion: false,
          shareScope: 'scope-b',
        },
      },
    }),
  ],
});

module.exports = [
  createConfig('analyzed', false),
  createConfig('disabled', true),
];
