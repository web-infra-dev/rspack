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
        // Raise A1/A2 to two requests so namedGroup keeps only B1/B2/C.
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
        // Alpha is not moved by namedGroup and must remain in this same-priority candidate.
        samePriority: {
          test: /[\\/](alpha|gamma)\.js$/,
          chunks: 'all',
          minChunks: 2,
          name: 'same_priority',
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
          const samePriority = compilation.namedChunks.get('same_priority');
          const alphaChunks = chunksContaining('alpha');
          const betaChunks = chunksContaining('beta');
          const gammaChunks = chunksContaining('gamma');

          expect(compilation.namedChunks.get('pre_split')).toBeDefined();
          expect(sharedNamed).toBeDefined();
          expect(samePriority).toBeDefined();
          expect(alphaChunks).toHaveLength(1);
          expect(alphaChunks).toContain(samePriority);
          expect(betaChunks).toHaveLength(1);
          expect(betaChunks).toContain(sharedNamed);
          expect(gammaChunks).toHaveLength(1);
          expect(gammaChunks).toContain(samePriority);
        });
      });
    },
  ],
};
