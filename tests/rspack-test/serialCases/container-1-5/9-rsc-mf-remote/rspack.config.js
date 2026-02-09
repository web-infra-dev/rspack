const path = require('node:path');
const { rspack, experiments } = require('@rspack/core');

const { ModuleFederationPlugin } = rspack.container;
const { createPlugins, Layers } = experiments.rsc;
const { ServerPlugin, ClientPlugin } = createPlugins();

const ssrEntry = path.join(__dirname, 'src/framework/entry.ssr.js');
const rscEntry = path.join(__dirname, 'src/framework/entry.rsc.js');
const clientEntry = path.join(__dirname, 'src/framework/entry.client.js');
const reactServerPath = path.join(
  path.dirname(require.resolve('react/package.json')),
  'react.react-server.js',
);

const swcLoaderRule = {
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
            },
          },
        },
        rspackExperiments: {
          reactServerComponents: true,
        },
      },
    },
  ],
};

function sharedByScope(layers) {
  return [
    {
      react: {
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
    },
    {
      react: {
        import: reactServerPath,
        shareKey: 'react',
        singleton: true,
        requiredVersion: false,
        shareScope: 'rsc',
        layer: layers.rsc,
        issuerLayer: layers.rsc,
      },
    },
  ];
}

/** @type {import('@rspack/core').Configuration[]} */
module.exports = [
  {
    name: 'remote-server',
    mode: 'development',
    target: 'async-node',
    context: __dirname,
    entry: {
      main: {
        import: ssrEntry,
      },
    },
    resolve: {
      extensions: ['...', '.jsx'],
    },
    output: {
      filename: '[name].js',
      chunkLoading: 'async-node',
      uniqueName: 'rsc-mf-remote-server',
    },
    module: {
      rules: [
        swcLoaderRule,
        {
          resource: ssrEntry,
          layer: Layers.ssr,
        },
        {
          resource: rscEntry,
          layer: Layers.rsc,
          resolve: {
            conditionNames: ['react-server', '...'],
          },
        },
        {
          issuerLayer: Layers.rsc,
          exclude: ssrEntry,
          resolve: {
            conditionNames: ['react-server', '...'],
          },
        },
      ],
    },
    plugins: [
      new ServerPlugin(),
      new rspack.DefinePlugin({
        CLIENT_PATH: JSON.stringify(path.resolve(__dirname, 'src/RemoteClient.js')),
      }),
      new ModuleFederationPlugin({
        name: 'rsc_remote_9',
        filename: 'remoteEntry.js',
        library: { type: 'commonjs-module' },
        runtimePlugins: [
          require.resolve('@module-federation/node/runtimePlugin'),
        ],
        exposes: {
          './App': {
            import: './src/RemoteApp.js',
            layer: Layers.rsc,
          },
        },
        shared: sharedByScope(Layers),
        experiments: {
          asyncStartup: true,
          rsc: true,
        },
      }),
    ],
  },
  {
    name: 'remote-client',
    mode: 'development',
    target: 'web',
    context: __dirname,
    entry: {
      main: {
        import: clientEntry,
      },
    },
    resolve: {
      extensions: ['...', '.jsx'],
    },
    output: {
      filename: 'client/[name].js',
      uniqueName: 'rsc-mf-remote-client',
    },
    module: {
      rules: [swcLoaderRule],
    },
    plugins: [new ClientPlugin()],
  },
];
