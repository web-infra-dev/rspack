const { ModuleFederationPlugin } = require('@rspack/core').container;

module.exports = [false, true].map((disableAssetsAnalyze) => {
  const name = disableAssetsAnalyze ? 'disabled' : 'analyzed';
  return {
    target: 'async-node',
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
        manifest: { fileName: `${name}.json`, disableAssetsAnalyze },
        exposes: { './consumer': './consumer.js' },
        shared: {
          './first.js': { shareKey: 'multiple', version: '1.0.0', requiredVersion: false },
          './second.js': { shareKey: 'multiple', version: '2.0.0', requiredVersion: false },
          single: { import: './single.js', version: '1.0.0', requiredVersion: false },
          'consumer-only': { import: false, requiredVersion: false },
        },
      }),
    ],
  };
});
