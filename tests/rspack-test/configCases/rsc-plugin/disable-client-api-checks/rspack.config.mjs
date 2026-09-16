import path from 'node:path';
import { experiments } from '@rspack/core';

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
          reactServerComponents: {
            disableClientApiChecks: true,
          },
        },
      },
    },
  ],
};

const cssRule = {
  test: /\.css$/,
  type: 'css/auto',
};

export default [
  {
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
      parser: {
        javascript: {
          exportsPresence: false,
        },
      },
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
    plugins: [new ServerPlugin()],
  },
  {
    name: 'client',
    mode: 'production',
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
      rules: [cssRule, swcLoaderRule],
    },
    plugins: [new ClientPlugin()],
  },
];
