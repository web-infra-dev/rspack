import { defineConfig } from '@rspack/cli';
import path from 'node:path';
import { experiments } from '@rspack/core';

const { createPlugins, Layers } = experiments.rsc;
const { ServerPlugin, ClientPlugin } = createPlugins();

const polyfillEntry = path.join(
  import.meta.dirname,
  'src/framework/polyfill.js',
);
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
    mode: 'production',
    target: 'node',
    entry: {
      main: {
        // The module that reaches the client component is not the entry's
        // first module.
        import: [polyfillEntry, ssrEntry],
      },
    },
    resolve: {
      extensions: ['...', '.ts', '.tsx', '.jsx'],
    },
    module: {
      rules: [
        swcLoaderRule,
        {
          resource: [polyfillEntry, ssrEntry],
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
      new ServerPlugin({
        onManifest(manifest) {
          expect(Object.keys(manifest)).toContain('main');
          const mainEntry = manifest.main;

          const clientPath = path.join(import.meta.dirname, './src/Client.js');
          const clientModuleId = './src/Client.js';
          expect(mainEntry.clientManifest[clientPath]).toBeDefined();
          expect(mainEntry.clientManifest[clientPath].id).toBe(clientModuleId);
          expect(mainEntry.serverConsumerModuleMap[clientModuleId]).toEqual({
            '*': {
              id: '(server-side-rendering)/./src/Client.js',
              name: '*',
              chunks: [],
              async: false,
            },
          });
        },
      }),
    ],
    optimization: {
      moduleIds: 'named',
      chunkIds: 'named',
    },
  },
  {
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
      rules: [swcLoaderRule],
    },
    plugins: [new ClientPlugin()],
    optimization: {
      moduleIds: 'named',
      chunkIds: 'named',
    },
  },
]);
