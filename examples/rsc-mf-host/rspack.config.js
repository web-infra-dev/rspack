import { createRequire } from 'node:module';
import { ModuleFederationPlugin } from '@module-federation/rspack';
import rspack from '@rspack/core';
import ReactRefreshPlugin from '@rspack/plugin-react-refresh';
import path from 'path';

const require = createRequire(import.meta.url);

const { createPlugins, Layers } = rspack.experiments.rsc;
const { ServerPlugin, ClientPlugin } = createPlugins();

const reactServerPath = path.join(
  path.dirname(require.resolve('react/package.json')),
  'react.react-server.js',
);
const sharedRscProbeShareKey = 'rsc-shared';
const sharedRscProbeRequest = 'rsc-shared';
const sharedRscProbeImport = 'rsc-shared';

const browserTargets = ['last 2 versions', '> 0.2%', 'not dead', 'Firefox ESR'];
const nodeTargets = ['node 22'];
const REMOTE_URL =
  process.env.RSC_REMOTE_URL || 'http://localhost:1717/server-mf-manifest.json';
const REMOTE_CLIENT_URL =
  process.env.RSC_REMOTE_CLIENT_URL ||
  'http://localhost:1717/static/mf-manifest.json';

function jsRule(targets) {
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
            experimental: {
              keepImportAttributes: true,
            },
          },
          env: { targets },
          rspackExperiments: {
            reactServerComponents: true,
          },
        },
      },
    ],
  };
}

function tsRule(targets) {
  return {
    test: /\.tsx?$/,
    use: [
      {
        loader: 'builtin:swc-loader',
        options: {
          jsc: {
            parser: {
              syntax: 'typescript',
              tsx: true,
            },
            transform: {
              react: {
                runtime: 'automatic',
              },
            },
            experimental: {
              keepImportAttributes: true,
            },
          },
          env: { targets },
          rspackExperiments: {
            reactServerComponents: true,
          },
        },
      },
    ],
  };
}

function cssRule() {
  return {
    test: /\.css$/i,
    type: 'css/auto',
  };
}

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
      [sharedRscProbeRequest]: {
        import: sharedRscProbeImport,
        shareKey: sharedRscProbeShareKey,
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
      'react-dom/server': {
        import: 'react-dom/server',
        shareKey: 'react-dom/server',
        singleton: true,
        requiredVersion: false,
        shareScope: 'ssr',
        layer: layers.ssr,
        issuerLayer: layers.ssr,
      },
      [sharedRscProbeRequest]: {
        import: sharedRscProbeImport,
        shareKey: sharedRscProbeShareKey,
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
      [sharedRscProbeRequest]: {
        import: sharedRscProbeImport,
        shareKey: sharedRscProbeShareKey,
        singleton: true,
        requiredVersion: false,
        shareScope: 'rsc',
        layer: layers.rsc,
        issuerLayer: layers.rsc,
      },
    },
  ];
}

function sharedClientByScope() {
  return {
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
    [sharedRscProbeRequest]: {
      import: sharedRscProbeImport,
      shareKey: sharedRscProbeShareKey,
      singleton: true,
      requiredVersion: false,
      shareScope: 'default',
    },
  };
}

function numericIdOptimization() {
  return {
    moduleIds: 'natural',
    chunkIds: 'natural',
  };
}

const SSR_ENTRY = path.resolve(
  import.meta.dirname,
  'src/framework/entry.ssr.tsx',
);
const RSC_ENTRY = path.resolve(
  import.meta.dirname,
  'src/framework/entry.rsc.tsx',
);

const state = { onServerComponentChanged: false };
export { state as rscState };

/** @type {import('@rspack/core').MultiCompilerOptions} */
export default [
  {
    name: 'client',
    mode: 'development',
    target: 'web',
    context: import.meta.dirname,
    entry: './src/framework/entry.client.tsx',
    resolve: {
      extensions: ['...', '.ts', '.tsx', '.jsx'],
    },
    output: {
      path: path.join(import.meta.dirname, 'dist/static'),
      publicPath: 'static/',
    },
    devtool: 'source-map',
    optimization: numericIdOptimization(),
    module: {
      rules: [cssRule(), jsRule(browserTargets), tsRule(browserTargets)],
    },
    plugins: [
      new ClientPlugin(),
      new ModuleFederationPlugin({
        name: 'rsc_host_client',
        manifest: false,
        remoteType: 'script',
        remotes: {
          rscRemote: {
            external: `rsc_remote_client@${REMOTE_CLIENT_URL}`,
            shareScope: 'default',
          },
        },
        shared: sharedClientByScope(),
        experiments: {
          asyncStartup: true,
          rsc: true,
        },
        dts: false,
      }),
      new rspack.HotModuleReplacementPlugin(),
      new ReactRefreshPlugin(),
    ],
  },
  {
    name: 'server',
    mode: 'development',
    target: 'async-node',
    context: import.meta.dirname,
    entry: './src/framework/entry.rsc.tsx',
    resolve: {
      extensions: ['...', '.ts', '.tsx', '.jsx'],
    },
    output: {
      path: path.join(import.meta.dirname, 'dist'),
      filename: '[name].cjs',
      chunkFilename: '[name].cjs',
      chunkLoading: 'async-node',
      library: {
        type: 'commonjs-module',
      },
    },
    devtool: false,
    optimization: numericIdOptimization(),
    module: {
      rules: [
        cssRule(),
        jsRule(nodeTargets),
        tsRule(nodeTargets),
        {
          resource: SSR_ENTRY,
          layer: Layers.ssr,
        },
        {
          resource: RSC_ENTRY,
          layer: Layers.rsc,
          resolve: {
            conditionNames: ['react-server', '...'],
          },
        },
        {
          issuerLayer: Layers.rsc,
          exclude: SSR_ENTRY,
          resolve: {
            conditionNames: ['react-server', '...'],
          },
        },
      ],
    },
    plugins: [
      new ServerPlugin({
        onServerComponentChanges() {
          state.onServerComponentChanged = true;
          console.log(
            '[RSC] server component changes detected, restarting server...',
          );
        },
      }),
      new ModuleFederationPlugin({
        name: 'rsc_host',
        manifest: false,
        filename: 'hostRemoteEntry.cjs',
        library: {
          type: 'commonjs-module',
        },
        remoteType: 'script',
        remotes: {
          rscRemote: {
            external: `rsc_remote@${REMOTE_URL}`,
            shareScope: ['ssr', 'rsc', 'default'],
          },
        },
        shared: sharedByScope(Layers),
        experiments: {
          asyncStartup: true,
          rsc: true,
        },
        dts: false,
      }),
    ],
    externalsType: 'commonjs',
    externals: {
      express: 'express',
    },
  },
];
