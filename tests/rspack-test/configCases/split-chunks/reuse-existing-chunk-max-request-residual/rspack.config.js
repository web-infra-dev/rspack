/** @typedef {import("@rspack/core").Compiler} Compiler */

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  target: 'web',
  optimization: {
    splitChunks: {
      minSize: 0,
      maxAsyncRequests: Infinity,
      cacheGroups: {
        preSplit: {
          test: /[\\/]prelude\.js$/,
          chunks: 'all',
          minChunks: 1,
          name: 'pre_split',
          priority: 200,
        },
        // Foo is at its request limit after preSplit, leaving only the reused destination.
        highPriority: {
          test: /[\\/]util\.js$/,
          chunks: 'all',
          maxAsyncRequests: 2,
          minChunks: 1,
          priority: 100,
          reuseExistingChunk: true,
        },
        // The reused destination is also claimed by highPriority, so this candidate must not move
        // util back out at the same priority.
        samePriority: {
          test: /[\\/]util\.js$/,
          chunks: 'all',
          minChunks: 1,
          name: 'same_priority',
          priority: 100,
        },
        lowerPriority: {
          test: /[\\/]util\.js$/,
          chunks: 'all',
          minChunks: 1,
          name: 'lower_util',
          priority: 0,
        },
        default: false,
        defaultVendors: false,
      },
    },
  },
  plugins: [
    /** @this {Compiler} */
    function () {
      this.hooks.compilation.tap(
        'ReuseExistingChunkMaxRequestResidual',
        (compilation) => {
          compilation.hooks.afterSeal.tap(
            'ReuseExistingChunkMaxRequestResidual',
            () => {
              const reusableChunk = compilation.namedChunks.get('ReusableUtil');
              const lowerPriorityChunk =
                compilation.namedChunks.get('lower_util');
              const samePriorityChunk =
                compilation.namedChunks.get('same_priority');
              const utilChunks = Array.from(compilation.chunks).filter(
                (chunk) =>
                  Array.from(
                    compilation.chunkGraph.getChunkModulesIterable(chunk),
                  ).some((module) =>
                    module
                      .identifier()
                      .replaceAll('\\', '/')
                      .endsWith('/util.js'),
                  ),
              );

              expect(reusableChunk).toBeDefined();
              expect(samePriorityChunk).toBeNull();
              expect(lowerPriorityChunk).toBeDefined();
              expect(utilChunks).toHaveLength(2);
              expect(utilChunks).toContain(reusableChunk);
              expect(utilChunks).toContain(lowerPriorityChunk);
            },
          );
        },
      );
    },
  ],
};
