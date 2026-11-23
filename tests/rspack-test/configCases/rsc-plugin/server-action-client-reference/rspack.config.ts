import { defineConfig } from '@rspack/cli';
import { experiments } from '@rspack/core';

const { createPlugins, Layers } = experiments.rsc;
const { ServerPlugin, ClientPlugin } = createPlugins();
const swc = {
  test: /\.js$/,
  loader: 'builtin:swc-loader',
  options: { rspackExperiments: { reactServerComponents: true } },
};

export default defineConfig([
  {
    mode: 'production',
    target: 'node',
    entry: { main: { import: './server.js', layer: Layers.rsc } },
    output: { filename: 'server-[name].js' },
    module: {
      rules: [
        swc,
        {
          issuerLayer: Layers.rsc,
          resolve: { conditionNames: ['react-server', '...'] },
        },
      ],
    },
    plugins: [new ServerPlugin()],
  },
  {
    mode: 'production',
    target: 'web',
    entry: './client.js',
    output: { filename: 'client-[name].js' },
    module: { rules: [swc] },
    plugins: [new ClientPlugin()],
  },
]);
