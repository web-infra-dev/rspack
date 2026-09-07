/** @typedef {import("@rspack/core").Compiler} Compiler */

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  target: 'web',
  optimization: {
    splitChunks: {
      minSize: 0,
      cacheGroups: {
        highPriority: {
          test: /[\\/]util\.js$/,
          chunks: /^(Foo|Bar|ReusableUtil)$/,
          minChunks: 2,
          priority: 100,
          reuseExistingChunk: true,
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
        'ReuseExistingChunkResidual',
        (compilation) => {
          compilation.hooks.afterSeal.tap('ReuseExistingChunkResidual', () => {
            const reusableChunk = compilation.namedChunks.get('ReusableUtil');
            const lowerPriorityChunk =
              compilation.namedChunks.get('lower_util');
            const utilChunks = Array.from(compilation.chunks).filter((chunk) =>
              Array.from(
                compilation.chunkGraph.getChunkModulesIterable(chunk),
              ).some((module) =>
                module.identifier().replaceAll('\\', '/').endsWith('/util.js'),
              ),
            );

            expect(reusableChunk).toBeDefined();
            expect(lowerPriorityChunk).toBeNull();
            expect(utilChunks).toHaveLength(1);
            expect(utilChunks).toContain(reusableChunk);
          });
        },
      );
    },
  ],
};
