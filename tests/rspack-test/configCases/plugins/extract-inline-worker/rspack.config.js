const { experiments } = require('@rspack/core');

const workerSource =
  'self.onmessage = event => postMessage("worker-result-marker:" + event.data);\n';

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'production',
  devtool: 'source-map',
  output: {
    publicPath: '/assets/',
  },
  plugins: [
    new experiments.ExtractInlineWorkerPlugin({
      filename: 'workers/[contenthash:10].js',
      minSize: 20,
    }),
    {
      apply(compiler) {
        compiler.hooks.done.tap('VerifyExtractedInlineWorker', (stats) => {
          const assets = stats.compilation.getAssets();
          const worker = assets.find((asset) =>
            /^workers\/.+\.js$/.test(asset.name),
          );
          const main = assets.find(
            (asset) =>
              asset.name.endsWith('.js') && !asset.name.startsWith('workers/'),
          );

          expect(worker).toBeDefined();
          expect(worker.source.source().toString()).toBe(workerSource);
          expect(main).toBeDefined();
          expect(main.source.source().toString()).toContain(
            `importScripts(\\"/assets/${worker.name}\\");`,
          );
          expect(main.source.source().toString()).not.toContain(
            'worker-result-marker',
          );
          expect(
            assets.some((asset) => asset.name === `${main.name}.map`),
          ).toBe(true);
        });
      },
    },
  ],
};
