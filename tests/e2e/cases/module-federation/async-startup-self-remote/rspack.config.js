const { rspack } = require('@rspack/core');

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
  context: __dirname,
  entry: './src/index.jsx',
  mode: 'development',
  devtool: false,
  resolve: {
    extensions: ['...', '.jsx'],
  },
  output: {
    filename: 'static/js/[name].js',
    chunkFilename: 'static/js/[name].js',
  },
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
  },
  module: {
    rules: [
      {
        test: /\.jsx?$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              jsc: {
                parser: {
                  syntax: 'ecmascript',
                  jsx: true,
                },
                transform: {
                  react: {
                    runtime: 'automatic',
                    development: true,
                  },
                },
              },
            },
          },
        ],
      },
    ],
  },
  plugins: [
    new rspack.HtmlRspackPlugin({ template: './src/index.html' }),
    function (compiler) {
      new rspack.container.ModuleFederationPlugin({
        name: 'remote',
        filename: 'remoteEntry.js',
        exposes: {
          './RemoteComponent': './src/RemoteComponent.jsx',
        },
        remotes: {
          remote: `remote@http://localhost:${compiler.options.devServer.port}/remoteEntry.js`,
        },
        shared: {
          react: {},
          'react-dom': {},
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
