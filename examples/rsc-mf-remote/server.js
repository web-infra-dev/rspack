import { createRequire } from 'node:module';
import rspack from '@rspack/core';
import ReactRefreshPlugin from '@rspack/plugin-react-refresh';
import express from 'express';
import path from 'path';
import webpackDevMiddleware from 'webpack-dev-middleware';
import webpackHotMiddleware from 'webpack-hot-middleware';
import { Worker } from 'worker_threads';

const require = createRequire(import.meta.url);

const { ModuleFederationPlugin } = rspack.container;
const { createPlugins, Layers } = rspack.experiments.rsc;
const { ServerPlugin, ClientPlugin } = createPlugins();

const SERVER_PORT = 1717;
const reactServerPath = path.join(
  path.dirname(require.resolve('react/package.json')),
  'react.react-server.js',
);

let hotMiddleware;
let onServerComponentChanged;
let currentWorker;
let workerRestartPromise;

const browserTargets = ['last 2 versions', '> 0.2%', 'not dead', 'Firefox ESR'];
const nodeTargets = ['node 22'];

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

const SSR_ENTRY = path.resolve(
  import.meta.dirname,
  'src/framework/entry.ssr.tsx',
);
const RSC_ENTRY = path.resolve(
  import.meta.dirname,
  'src/framework/entry.rsc.tsx',
);

const rspackConfig = [
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
    module: {
      rules: [cssRule(), jsRule(browserTargets), tsRule(browserTargets)],
    },
    plugins: [
      new ClientPlugin(),
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
          onServerComponentChanged = true;
          console.log(
            '[RSC] server component changes detected, restarting server...',
          );
        },
      }),
      new ModuleFederationPlugin({
        name: 'rsc_remote',
        filename: 'remoteEntry.cjs',
        library: {
          type: 'commonjs-module',
        },
        exposes: {
          './Todos': {
            import: './src/Todos.tsx',
            layer: Layers.rsc,
          },
          './RemoteShell': {
            import: './src/RemoteShell.tsx',
            layer: Layers.rsc,
          },
          './TodoList': {
            import: './src/TodoList.tsx',
            layer: Layers.rsc,
          },
          './TodoDetail': {
            import: './src/TodoDetail.tsx',
            layer: Layers.rsc,
          },
          './TodoCreate': {
            import: './src/TodoCreate.tsx',
            layer: Layers.rsc,
          },
          './TodoItem': {
            import: './src/TodoItem.tsx',
            layer: Layers.rsc,
          },
          './Dialog': {
            import: './src/Dialog.tsx',
            layer: Layers.rsc,
          },
          './Actions': {
            import: './src/actions.ts',
            layer: Layers.rsc,
          },
          './ServerOnly': {
            import: './src/serverOnly.ts',
            layer: Layers.rsc,
          },
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
    externalsType: 'commonjs',
    externals: {
      express: 'express',
    },
  },
];

const compiler = rspack(rspackConfig);

compiler.compilers[1].hooks.done.tapPromise('RestartWorker', async (stats) => {
  if (stats.hasErrors()) {
    console.error('[Server] Build failed with errors');
    return;
  }

  workerRestartPromise = (async () => {
    if (currentWorker) {
      await currentWorker.terminate();
      currentWorker = null;
    }

    currentWorker = await createServerWorker();
    if (onServerComponentChanged) {
      hotMiddleware.publish({ type: 'rsc:update' });
    }
    onServerComponentChanged = false;
  })();

  await workerRestartPromise;
});

compiler.compilers[0].hooks.done.tapPromise('WaitForWorker', async () => {
  if (!workerRestartPromise) return;
  try {
    await workerRestartPromise;
    await new Promise((resolve) => setTimeout(resolve, 1000));
  } catch (_error) {
    // no-op in dev server orchestration
  }
});

const app = express();

app.use(
  webpackDevMiddleware(compiler, {
    writeToDisk: true,
  }),
);

hotMiddleware = webpackHotMiddleware(compiler.compilers[0], {
  log: console.log,
  path: '/__rspack_hmr',
  heartbeat: 10 * 1000,
});
app.use(hotMiddleware);

function createServerWorker() {
  return new Promise((resolve, reject) => {
    const workerPath = path.join(import.meta.dirname, 'dist/main.cjs');
    const worker = new Worker(workerPath);

    worker.on('message', (message) => {
      if (message.type === 'ready') {
        resolve(worker);
      }
    });

    worker.on('error', (error) => {
      reject(error);
    });

    worker.on('exit', (code) => {
      if (code !== 0) {
        reject(new Error(`Worker stopped with exit code ${code}`));
      }
    });

    setTimeout(() => {
      reject(new Error('Worker initialization timeout'));
    }, 10000);
  });
}

const server = app.listen(SERVER_PORT, 'localhost', function () {
  console.log('Remote dev server is running on %j', server.address());
});
