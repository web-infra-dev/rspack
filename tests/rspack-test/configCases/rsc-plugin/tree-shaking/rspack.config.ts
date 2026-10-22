import { defineConfig } from '@rspack/cli';
import path from 'node:path';
import { rspack, experiments } from '@rspack/core';

const { createPlugins, Layers } = experiments.rsc;
const { ServerPlugin, ClientPlugin } = createPlugins();

const ssrEntry = path.join(import.meta.dirname, 'src/framework/entry.ssr.js');
const rscEntry = path.join(import.meta.dirname, 'src/framework/entry.rsc.js');

const swcLoaderRule = {
  test: /\.jsx?$/,
  use: [
    {
      loader: 'builtin:swc-loader',
      options: {
        detectSyntax: 'auto',
        jsc: {
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

const cssRule = {
  test: /\.css$/,
  type: 'css/auto',
};

export default defineConfig([
  {
    externals: {
      './static/main.js': 'commonjs ./static/main.js',
    },
    name: 'server',
    mode: 'production',
    target: 'node',
    entry: {
      main: {
        import: ssrEntry,
      },
    },
    resolve: {
      extensions: ['...', '.ts', '.tsx', '.jsx'],
    },
    module: {
      rules: [
        cssRule,
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
          resolve: {
            conditionNames: ['react-server', '...'],
          },
        },
      ],
    },
    plugins: [
      new ServerPlugin(),
      new rspack.DefinePlugin({
        DYNAMIC_TODO_PATH: JSON.stringify(
          path.join(import.meta.dirname, 'src/DynamicTodo.js'),
        ),
        TODOS_PATH: JSON.stringify(
          path.join(import.meta.dirname, 'src/Todos.js'),
        ),
      }),
    ],
    output: {
      filename: '[name].js',
    },
  },
  {
    externals: {
      './static/main.js': 'commonjs ./static/main.js',
    },
    name: 'client',
    mode: 'production',
    target: 'node',
    entry: {
      main: {
        import: './src/framework/entry.client.js',
      },
    },
    resolve: {
      extensions: ['...', '.ts', '.tsx', '.jsx'],
    },
    module: {
      rules: [cssRule, swcLoaderRule],
    },
    plugins: [new ClientPlugin()],
    output: {
      filename: 'static/[name].js',
      library: {
        type: 'commonjs',
      },
    },
  },
]);
