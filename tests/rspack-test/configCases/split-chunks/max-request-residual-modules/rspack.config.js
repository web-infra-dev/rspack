/** @typedef {import("@rspack/core").Compiler} Compiler */

/**
 * @param {string} name
 * @param {number} namedGroupMinSize
 * @param {number} namedGroupMinSizeReduction
 * @param {boolean} expectResidualSplit
 * @returns {import("@rspack/core").Configuration}
 */
function createConfig(
  name,
  namedGroupMinSize,
  namedGroupMinSizeReduction,
  expectResidualSplit,
) {
  const cacheGroups = {
    // Raise A1/A2 to two requests. D keeps one alpha source below minChunks in the residual-split
    // variant, while the size-validation variants exclude D and keep only B1/B2/C.
    preSplit: {
      test: /[\\/]prelude\.js$/,
      chunks: 'all',
      minChunks: 2,
      name: 'pre_split',
      priority: 200,
    },
    namedGroup: {
      test: /[\\/](alpha|beta)\.js$/,
      chunks: expectResidualSplit ? 'all' : (chunk) => chunk.name !== 'D',
      minChunks: 2,
      minSize: namedGroupMinSize,
      minSizeReduction: namedGroupMinSizeReduction,
      maxAsyncRequests: 2,
      name: 'shared_named',
      priority: 100,
    },
    defaultVendors: false,
  };

  if (expectResidualSplit) {
    // Alpha is not moved by namedGroup and must remain in this same-priority candidate.
    cacheGroups.samePriority = {
      test: /[\\/](alpha|gamma)\.js$/,
      chunks: 'all',
      minChunks: 2,
      name: 'same_priority',
      priority: 100,
    };
    cacheGroups.default = {
      chunks: 'all',
      minChunks: 2,
    };
  } else {
    cacheGroups.default = false;
  }

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
        cacheGroups,
      },
    },
    plugins: [
      /** @this {Compiler} */
      function () {
        this.hooks.compilation.tap(name, (compilation) => {
          compilation.hooks.afterSeal.tap(name, () => {
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

            if (expectResidualSplit) {
              expect(sharedNamed).toBeDefined();
              expect(samePriority).toBeDefined();
              expect(alphaChunks).toHaveLength(1);
              expect(alphaChunks).toContain(samePriority);
              expect(betaChunks).toHaveLength(1);
              expect(betaChunks).toContain(sharedNamed);
              expect(gammaChunks).toHaveLength(1);
              expect(gammaChunks).toContain(samePriority);
            } else {
              // Only beta remains after max-request pruning, and beta alone violates the configured
              // size constraint.
              expect(sharedNamed).toBeNull();
              expect(alphaChunks).toHaveLength(3);
              expect(betaChunks).toHaveLength(3);
              expect(gammaChunks).toHaveLength(2);
            }
          });
        });
      },
    ],
  };
}

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  createConfig('actual-moved-modules', 0, 0, true),
  createConfig('revalidate-min-size', 30, 0, false),
  createConfig('revalidate-min-size-reduction', 0, 100, false),
];
