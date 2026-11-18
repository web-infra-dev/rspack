import path from 'node:path';
import { promisify } from 'node:util';
import { experiments, rspack } from '@rspack/core';

const { Layers, createPlugins } = experiments.rsc;

export default {
  mode: 'development',
  module: {
    rules: [{ test: /\.css$/, type: 'css', generator: { exportsOnly: false } }],
  },
  output: { cssChunkFilename: 'clients.css' },
  plugins: [
    (compiler) => {
      const { ServerPlugin, ClientPlugin } = createPlugins();
      new ClientPlugin().apply(compiler);

      // The client compiler gets its entry dependencies from the server build.
      let serverBuild;
      compiler.hooks.beforeRun.tap('RscEntryTest', () => {
        serverBuild = promisify(rspack)({
          context: compiler.context,
          mode: 'development',
          target: 'node',
          entry: { main: { import: './page.js', layer: Layers.rsc } },
          output: { path: path.join(compiler.outputPath, 'server') },
          resolve: { conditionNames: ['react-server', '...'] },
          module: {
            rules: [
              {
                test: /\.js$/,
                loader: 'builtin:swc-loader',
                options: { rspackExperiments: { reactServerComponents: true } },
              },
            ],
          },
          plugins: [new ServerPlugin()],
        });
      });
      compiler.hooks.done.tapPromise('RscEntryTest', async () => {
        const stats = await serverBuild;
        expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
      });
    },
  ],
};
