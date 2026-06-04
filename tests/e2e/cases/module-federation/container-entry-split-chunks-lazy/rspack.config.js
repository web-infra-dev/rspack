const { rspack } = require('@rspack/core');
const { ReactRefreshRspackPlugin } = require('@rspack/plugin-react-refresh');

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
  context: __dirname,
  entry: './src/index.jsx',
  mode: 'development',
  devtool: false,
  lazyCompilation: { entries: true },
  resolve: {
    extensions: ['...', '.jsx'],
  },
  module: {
    rules: [
      {
        test: /\.(jsx?|tsx?)$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              detectSyntax: 'auto',
              jsc: {
                transform: {
                  react: {
                    runtime: 'automatic',
                    development: true,
                    refresh: true,
                  },
                },
              },
            },
          },
        ],
      },
    ],
  },
  optimization: {
    // host (main) and the container (remoteEntry) run on the same page and would otherwise
    // each emit their own runtime, clashing on the shared `self["rspackHotUpdate"]` global.
    // A single shared runtime keeps one HMR global, so lazyCompilation entry activation
    // (delivered over HMR) works in this self-referential MF setup. See #12443.
    runtimeChunk: 'single',
    splitChunks: {
      chunks: 'all',
      minSize: 0, // ensure dev server and hmr client is splitted into vendor chunk
    },
  },
  plugins: [
    new rspack.HtmlRspackPlugin({ template: './src/index.html' }),
    new ReactRefreshRspackPlugin(),
    function (compiler) {
      new rspack.container.ModuleFederationPluginV1({
        name: 'remote',
        filename: 'remoteEntry.js',
        exposes: {
          './Component': './src/RemoteComponent.jsx',
        },
        remotes: {
          remote: `remote@http://localhost:${compiler.options.devServer.port}/remoteEntry.js`,
        },
        shared: {
          react: {},
          'react-dom': {},
        },
      }).apply(compiler);
    },
  ],
  devServer: {
    hot: true,
    port: 8080,
    devMiddleware: {
      writeToDisk: true,
    },
  },
};
