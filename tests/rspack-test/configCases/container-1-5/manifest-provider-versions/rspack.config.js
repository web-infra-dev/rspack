const { ModuleFederationPlugin } = require('@rspack/core').container;

const shares = {
  one: { import: './one.js', shareKey: 'versions', version: '1.0.0' },
  two: { import: './two.js', shareKey: 'versions', version: '2.0.0' },
  unknown: { import: false, shareKey: 'versions' },
  default: { import: './one.js', shareKey: 'scopes', version: '1.0.0' },
  custom: {
    import: './two.js',
    shareKey: 'scopes',
    version: '1.0.0',
    shareScope: 'custom',
  },
  differentVersion: {
    import: './two.js',
    shareKey: 'scopes',
    version: '2.0.0',
  },
  differentLayer: {
    import: './one.js',
    shareKey: 'scopes',
    version: '1.0.0',
    layer: 'server',
  },
  uniqueConsumer: { import: false, shareKey: 'unique' },
  uniqueProvider: { import: './one.js', shareKey: 'unique', version: '1.0.0' },
};

module.exports = ['forward', 'reverse'].map((name, index) => ({
  target: 'async-node',
  experiments: { layers: true },
  output: { filename: name + '/[name].js', chunkFilename: name + '/[id].js' },
  plugins: [
    new ModuleFederationPlugin({
      implementation: require.resolve('@module-federation/runtime-tools-layers'),
      name: 'provider_versions',
      manifest: { fileName: name + '.json', disableAssetsAnalyze: true },
      shared: Object.fromEntries(
        (index ? Object.entries(shares).reverse() : Object.entries(shares)).map(
          ([request, options]) => [
            request,
            { ...options, requiredVersion: false },
          ],
        ),
      ),
    }),
  ],
}));
