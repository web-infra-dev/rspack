const { SubresourceIntegrityPlugin, container } = require("@rspack/core");

module.exports = {
  mode: 'production',
  target: ["web", "es2017"],
  output: {
    filename: '[name].[contenthash:8].js',
    chunkFilename: '[name].[contenthash:8].js',
    assetModuleFilename: '[name].[contenthash:8][ext]',
    uniqueName: 'main_app',
    crossOriginLoading: "anonymous"
  },
  resolve: {
    alias: {
      path: false,
      "node:path": false
    }
  },
  optimization: {
    minimize: false,
    splitChunks: {
      chunks: 'all',
    },
  },
  performance: false,
  plugins: [
    new SubresourceIntegrityPlugin(),
    new container.ModuleFederationPlugin({
      name: "main_app",
      exposes: {
        "./mf": {
          import: ["./mf-expose"],
          name: '__federation_expose_mf'
        },
      },
    })
  ],
};
