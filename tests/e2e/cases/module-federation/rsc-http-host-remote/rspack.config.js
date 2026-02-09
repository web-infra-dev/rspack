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
  entry: './src/host/index.jsx',
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
        test: /src\/host\/App\.jsx$/,
        layer: LAYERS.rsc,
      },
      {
        test: /src\/host\/SsrProbe\.js$/,
        layer: LAYERS.ssr,
      },
    ],
  },
  output: {
    filename: 'static/js/[name].js',
    chunkFilename: 'static/js/[name].js',
    uniqueName: 'rsc_mf_e2e_host',
  },
  plugins: [
    new rspack.HtmlRspackPlugin({ template: './src/host/index.html' }),
    function applyModuleFederation(compiler) {
      const hostPort = Number(compiler.options.devServer.port);
      const remotePort = hostPort + 100;
      new ModuleFederationPlugin({
        name: 'rsc_host_e2e',
        remotes: {
          rscRemote: `rsc_remote_e2e@http://localhost:${remotePort}/remoteEntry.js`,
        },
        shared: sharedByScope(LAYERS),
        experiments: {
          asyncStartup: true,
          rsc: true,
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
