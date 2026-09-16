const { ModuleFederationPlugin } = require('@rspack/core').container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
  },
  experiments: {
    layers: true,
  },
  output: {
    chunkFilename: '[id].js',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      filename: 'container.[chunkhash:8].js',
      library: { type: 'commonjs-module' },
      exposes: {
        'expose-a': {
          import: './module.js',
          layer: 'server',
        },
      },
      remoteType: 'script',
      remotes: {
        remote: 'remote@http://localhost:8000/remoteEntry.js',
      },
      shared: {
        react: {
          layer: 'server',
          shareScope: ['server', 'default'],
        },
      },
      manifest: {
        disableAssetsAnalyze: true,
      },
    }),
  ],
};
