const { rspack } = require('@rspack/core');

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
  context: __dirname,
  entry: './src/index.js',
  mode: 'development',
  devtool: false,
  resolve: {
    extensions: ['...'],
  },
  output: {
    filename: 'static/js/[name].js',
    chunkFilename: 'static/js/[name].js',
  },
  optimization: {
    runtimeChunk: 'single',
    chunkIds: 'named',
    moduleIds: 'named',
  },
  plugins: [
    new rspack.HtmlRspackPlugin({ template: './src/index.html' }),
    function (compiler) {
      new rspack.container.ModuleFederationPlugin({
        name: 'remote',
        filename: 'remoteEntry.js',
        exposes: {
          './RemoteModule': './src/RemoteModule.js',
        },
        remotes: {
          remote: `remote@http://localhost:${compiler.options.devServer.port}/remoteEntry.js`,
        },
        experiments: {
          asyncStartup: true,
        },
      }).apply(compiler);
    },
  ],
  devServer: {
    hot: true,
    devMiddleware: {
      writeToDisk: true,
    },
  },
};
