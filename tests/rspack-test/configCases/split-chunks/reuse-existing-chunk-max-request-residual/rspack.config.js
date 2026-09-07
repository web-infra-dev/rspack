/** @typedef {import("@rspack/core").Compiler} Compiler */

/**
 * @param {string} name
 * @param {number} minSizeReduction
 * @param {boolean} expectSamePriority
 * @returns {import("@rspack/core").Configuration}
 */
function createConfig(name, minSizeReduction, expectSamePriority) {
  return {
    name,
    mode: 'development',
    target: 'web',
    output: {
      filename: `${name}-[name].js`,
      chunkFilename: `${name}-[name].js`,
    },
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
            minSizeReduction,
            priority: 100,
            reuseExistingChunk: true,
          },
          // With no size-reduction requirement, the reused destination is claimed by highPriority
          // and this candidate must not move util back out at the same priority. With a positive
          // requirement, highPriority reduces no source chunk and this candidate must remain valid.
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
        this.hooks.compilation.tap(name, (compilation) => {
          compilation.hooks.afterSeal.tap(name, () => {
            const reusableChunk = compilation.namedChunks.get('ReusableUtil');
            const lowerPriorityChunk =
              compilation.namedChunks.get('lower_util');
            const samePriorityChunk =
              compilation.namedChunks.get('same_priority');
            const utilChunks = Array.from(compilation.chunks).filter((chunk) =>
              Array.from(
                compilation.chunkGraph.getChunkModulesIterable(chunk),
              ).some((module) =>
                module.identifier().replaceAll('\\', '/').endsWith('/util.js'),
              ),
            );

            expect(reusableChunk).toBeDefined();
            if (expectSamePriority) {
              expect(samePriorityChunk).toBeDefined();
              expect(lowerPriorityChunk).toBeNull();
              expect(utilChunks).toHaveLength(1);
              expect(utilChunks).toContain(samePriorityChunk);
            } else {
              expect(samePriorityChunk).toBeNull();
              expect(lowerPriorityChunk).toBeDefined();
              expect(utilChunks).toHaveLength(2);
              expect(utilChunks).toContain(reusableChunk);
              expect(utilChunks).toContain(lowerPriorityChunk);
            }
          });
        });
      },
    ],
  };
}

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  createConfig('preserve-reused-destination', 0, false),
  createConfig('revalidate-min-size-reduction', 1, true),
];
