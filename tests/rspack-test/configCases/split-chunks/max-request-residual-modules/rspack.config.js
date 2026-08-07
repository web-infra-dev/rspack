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
        // Raise A1/A2 to two requests so namedGroup keeps only B1/B2.
        preSplit: {
          test: /[\\/]prelude\.js$/,
          chunks: 'all',
          minChunks: 2,
          name: 'pre_split',
          priority: 200,
        },
        namedGroup: {
          test: /[\\/](alpha|beta)\.js$/,
          chunks: 'all',
          minChunks: 2,
          maxAsyncRequests: 2,
          name: 'shared_named',
          priority: 100,
        },
        default: {
          chunks: 'all',
          minChunks: 2,
        },
        defaultVendors: false,
      },
    },
  },
  plugins: [
    /** @this {Compiler} */
    function () {
      this.hooks.compilation.tap('MaxRequestResidualModules', (compilation) => {
        compilation.hooks.afterSeal.tap('MaxRequestResidualModules', () => {
          const chunksContaining = (moduleName) =>
            Array.from(compilation.chunks).filter((chunk) =>
              Array.from(
                compilation.chunkGraph.getChunkModulesIterable(chunk),
              ).some((module) =>
                module
                  .identifier()
                  .replaceAll('\\', '/')
                  .endsWith(`/${moduleName}.js`),
              ),
            );

          const sharedNamed = compilation.namedChunks.get('shared_named');
          const alphaChunks = chunksContaining('alpha');
          const betaChunks = chunksContaining('beta');

          expect(compilation.namedChunks.get('pre_split')).toBeDefined();
          expect(sharedNamed).toBeDefined();
          expect(alphaChunks).toHaveLength(1);
          expect(alphaChunks).not.toContain(sharedNamed);
          expect(betaChunks).toEqual([sharedNamed]);
        });
      });
    },
  ],
};
