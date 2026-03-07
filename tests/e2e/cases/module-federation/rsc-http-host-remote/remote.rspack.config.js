const { rspack } = require('@rspack/core');

const { ModuleFederationPlugin } = rspack.container;
const LAYERS = {
  ssr: 'server-side-rendering',
  rsc: 'react-server-components',
};

function sharedByScope(layers) {
  return [
    {
      react: {
        singleton: true,
        requiredVersion: false,
        shareScope: 'default',
      },
      'react-dom': {
        singleton: true,
        requiredVersion: false,
        shareScope: 'default',
      },
    },
    {
      react: {
        import: 'react',
        shareKey: 'react',
        singleton: true,
        requiredVersion: false,
        shareScope: 'ssr',
        layer: layers.ssr,
        issuerLayer: layers.ssr,
      },
      'react-dom': {
        import: 'react-dom',
        shareKey: 'react-dom',
        singleton: true,
        requiredVersion: false,
        shareScope: 'ssr',
        layer: layers.ssr,
        issuerLayer: layers.ssr,
      },
    },
    {
      react: {
        import: 'react',
        shareKey: 'react',
        singleton: true,
        requiredVersion: false,
        shareScope: 'rsc',
        layer: layers.rsc,
        issuerLayer: layers.rsc,
      },
      'react-dom': {
        import: 'react-dom',
        shareKey: 'react-dom',
        singleton: true,
        requiredVersion: false,
        shareScope: 'rsc',
        layer: layers.rsc,
        issuerLayer: layers.rsc,
      },
    },
  ];
}

/** @type {import('@rspack/core').RspackOptions} */
module.exports = {
  context: __dirname,
  mode: 'development',
  devtool: false,
  entry: './src/remote/entry.js',
  resolve: {
    extensions: ['...', '.jsx'],
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
      {
        test: /src\/remote\/(RemoteWidget|LayerInfo)\.jsx?$/,
        layer: LAYERS.rsc,
      },
    ],
  },
  output: {
    filename: 'static/js/[name].js',
    chunkFilename: 'static/js/[name].js',
    uniqueName: 'rsc_mf_e2e_remote',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'rsc_remote_e2e',
      filename: 'remoteEntry.js',
      exposes: {
        './RemoteWidget': {
          import: './src/remote/RemoteWidget.jsx',
          layer: LAYERS.rsc,
        },
        './LayerInfo': {
          import: './src/remote/LayerInfo.js',
          layer: LAYERS.rsc,
        },
      },
      shared: sharedByScope(LAYERS),
      experiments: {
        asyncStartup: true,
        rsc: true,
      },
    }),
  ],
  devServer: {
    hot: true,
    devMiddleware: {
      writeToDisk: true,
    },
  },
};
