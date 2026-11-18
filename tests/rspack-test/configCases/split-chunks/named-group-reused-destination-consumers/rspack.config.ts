import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  target: 'web',
  output: {
    chunkFilename: '[name].js',
  },
  optimization: {
    splitChunks: {
      minSize: 0,
      minSizeReduction: 0,
      maxAsyncRequests: Infinity,
      cacheGroups: {
        preSplit: {
          test: /[\\/]prelude\.js$/,
          chunks: 'all',
          minChunks: 1,
          name: 'pre_split',
          priority: 200,
        },
        // Each consumer is at its request limit after preSplit.
        highPriority: {
          test: /[\\/]util\.js$/,
          chunks: 'all',
          maxAsyncRequests: 2,
          minChunks: 1,
          minSizeReduction: 0,
          priority: 100,
          reuseExistingChunk: true,
        },
        samePriority: {
          test: /[\\/]util\.js$/,
          chunks: 'all',
          minChunks: 1,
          name: 'same_priority',
          priority: 100,
        },
        default: false,
        defaultVendors: false,
      },
    },
  },
  plugins: [
    definePlugin(function () {
      this.hooks.compilation.tap('SharedConsumerUtil', (compilation) => {
        compilation.hooks.afterSeal.tap('SharedConsumerUtil', () => {
          const utilChunks = Array.from(compilation.chunks).filter((chunk) =>
            Array.from(
              compilation.chunkGraph.getChunkModulesIterable(chunk),
            ).some((module) =>
              module.identifier().replaceAll('\\', '/').endsWith('/util.js'),
            ),
          );
          for (const name of ['Foo', 'Bar']) {
            const consumer = compilation.namedChunks.get(name);
            expect(consumer).toBeDefined();
            expect(utilChunks).not.toContain(consumer);
            const shared = utilChunks.filter((chunk) =>
              Array.from(chunk.groupsIterable).some(
                (group) => group.name === name,
              ),
            );
            expect(shared.map((chunk) => chunk.name)).toEqual([
              'same_priority',
            ]);
          }
        });
      });
    }),
  ],
});
