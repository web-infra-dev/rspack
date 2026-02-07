const path = require('node:path');
const { rspack, experiments } = require('@rspack/core');

const { ModuleFederationPlugin } = rspack.container;
const { createPlugins, Layers } = experiments.rsc;
const reactServerPath = path.join(
  path.dirname(require.resolve('react/package.json')),
  'react.react-server.js',
);
const REMOTE_PORT = 13010;

function createSwcRule() {
  return {
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
}

function createRscLayerRules({ ssrEntry, rscEntry, layers }) {
  return [
    {
      resource: ssrEntry,
      layer: layers.ssr,
    },
    {
      resource: rscEntry,
      layer: layers.rsc,
      resolve: {
        conditionNames: ['react-server', '...'],
      },
    },
    {
      issuerLayer: layers.rsc,
      exclude: ssrEntry,
      resolve: {
        conditionNames: ['react-server', '...'],
      },
    },
  ];
}

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

const remoteSsrEntry = path.join(__dirname, 'remote/src/framework/entry.ssr.js');
const remoteRscEntry = path.join(__dirname, 'remote/src/framework/entry.rsc.js');
const remoteClientEntry = path.join(__dirname, 'remote/src/framework/entry.client.js');

const hostSsrEntry = path.join(__dirname, 'host/src/framework/entry.ssr.js');
const hostRscEntry = path.join(__dirname, 'host/src/framework/entry.rsc.js');
const hostClientEntry = path.join(__dirname, 'host/src/framework/entry.client.js');

const remotePlugins = createPlugins();
const hostPlugins = createPlugins();

/** @type {import('@rspack/core').Configuration[]} */
module.exports = [
  {
    name: 'remote-server',
    mode: 'development',
    target: 'async-node',
    context: __dirname,
    entry: {
      main: {
        import: remoteSsrEntry,
      },
    },
    resolve: {
      extensions: ['...', '.jsx'],
    },
    output: {
      filename: 'remote/[name].js',
      chunkFilename: 'remote/[name].js',
      chunkLoading: 'async-node',
      uniqueName: 'rsc-mf-http-remote-server',
    },
    module: {
      rules: [
        createSwcRule(),
        ...createRscLayerRules({
          ssrEntry: remoteSsrEntry,
          rscEntry: remoteRscEntry,
          layers: Layers,
        }),
      ],
    },
    plugins: [
      new remotePlugins.ServerPlugin(),
      new rspack.DefinePlugin({
        CLIENT_PATH: JSON.stringify(
          path.resolve(__dirname, 'remote/src/RemoteClient.js'),
        ),
      }),
      new ModuleFederationPlugin({
        name: 'rsc_remote_render',
        filename: 'remote/remoteEntry.js',
        library: { type: 'commonjs-module' },
        runtimePlugins: [
          require.resolve('@module-federation/node/runtimePlugin'),
        ],
        exposes: {
          './App': {
            import: './remote/src/RemoteApp.js',
            layer: Layers.rsc,
          },
          './ClientWidget': {
            import: './remote/src/RemoteClient.js',
            layer: Layers.rsc,
          },
          './ServerOnlyInfo': {
            import: './remote/src/ServerOnlyInfo.js',
            layer: Layers.rsc,
          },
          './Actions': {
            import: './remote/src/actions.js',
            layer: Layers.rsc,
          },
          './NestedActions': {
            import: './remote/src/nestedActions.js',
            layer: Layers.rsc,
          },
          './ServerComponent': {
            import: './remote/src/RemoteServerCard.js',
            layer: Layers.rsc,
          },
          './NestedMixed': {
            import: './remote/src/RemoteNestedMixed.js',
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
        import: remoteClientEntry,
      },
    },
    resolve: {
      extensions: ['...', '.jsx'],
    },
    output: {
      filename: 'remote-client/[name].js',
      uniqueName: 'rsc-mf-http-remote-client',
    },
    module: {
      rules: [createSwcRule()],
    },
    plugins: [new remotePlugins.ClientPlugin()],
  },
  {
    name: 'host-server',
    mode: 'development',
    target: 'async-node',
    context: __dirname,
    entry: {
      main: {
        import: hostSsrEntry,
      },
    },
    resolve: {
      extensions: ['...', '.jsx'],
    },
    output: {
      filename: 'host/[name].js',
      chunkFilename: 'host/[name].js',
      chunkLoading: 'async-node',
      uniqueName: 'rsc-mf-http-host-server',
    },
    module: {
      rules: [
        createSwcRule(),
        ...createRscLayerRules({
          ssrEntry: hostSsrEntry,
          rscEntry: hostRscEntry,
          layers: Layers,
        }),
      ],
    },
    plugins: [
      new hostPlugins.ServerPlugin(),
      new ModuleFederationPlugin({
        name: 'rsc_host_render',
        filename: 'host/hostRemoteEntry.js',
        library: { type: 'commonjs-module' },
        remoteType: 'script',
        remotes: {
          rscRemote: `rsc_remote_render@http://127.0.0.1:${REMOTE_PORT}/remote/remoteEntry.js`,
        },
        runtimePlugins: [
          require.resolve('@module-federation/node/runtimePlugin'),
        ],
        shared: sharedByScope(Layers),
        experiments: {
          asyncStartup: true,
          rsc: true,
        },
      }),
    ],
  },
  {
    name: 'host-client',
    mode: 'development',
    target: 'web',
    context: __dirname,
    entry: {
      main: {
        import: hostClientEntry,
      },
    },
    resolve: {
      extensions: ['...', '.jsx'],
    },
    output: {
      filename: 'host-client/[name].js',
      uniqueName: 'rsc-mf-http-host-client',
    },
    module: {
      rules: [createSwcRule()],
    },
    plugins: [new hostPlugins.ClientPlugin()],
  },
];
