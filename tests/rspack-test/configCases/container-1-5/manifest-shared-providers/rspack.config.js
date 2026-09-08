const { ModuleFederationPlugin } = require('@rspack/core').container;

const createConfig = (name, identity) => ({
  target: 'async-node',
  experiments: { layers: true },
  optimization: { chunkIds: 'named', moduleIds: 'named' },
  output: {
    filename: `${name}/[name].js`,
    chunkFilename: `${name}/[id].js`,
    uniqueName: `shared-providers-${name}`,
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'shared_providers',
      filename: `${name}/container.js`,
      manifest: { fileName: `${name}.json` },
      exposes: { './consumer': './consumer.js' },
      shared: {
        './first': {
          shareKey: 'multiple',
          treeShaking: { mode: 'runtime-infer' },
          ...identity,
          version: '1.0.0',
          requiredVersion: false,
        },
        './second.js': {
          shareKey: 'multiple',
          treeShaking: { mode: 'runtime-infer' },
          ...identity,
          version: '2.0.0',
          requiredVersion: false,
        },
        single: {
          import: './first.js',
          version: '1.0.0',
          requiredVersion: false,
        },
        'consumer-only': { import: false, requiredVersion: false },
      },
    }),
  ],
});

module.exports = [
  createConfig('layered', { shareScope: 'custom', layer: 'server' }),
  createConfig('default', {}),
];
