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

export default defineConfig([
  {
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
        CLIENT_PATH: JSON.stringify(
          path.resolve(import.meta.dirname, 'src/Client.js'),
        ),
      }),
    ],
    optimization: {
      moduleIds: 'named',
      concatenateModules: true,
    },
  },
  {
    target: 'web',
    entry: {
      main: {
        import: './src/framework/entry.client.js',
      },
    },
    resolve: {
      extensions: ['...', '.ts', '.tsx', '.jsx'],
    },
    module: {
      rules: [swcLoaderRule],
    },
    plugins: [new ClientPlugin()],
    optimization: {
      moduleIds: 'named',
      concatenateModules: true,
    },
  },
]);
