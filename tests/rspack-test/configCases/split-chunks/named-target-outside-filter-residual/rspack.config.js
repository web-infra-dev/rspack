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
        highPriority: {
          test: /[\\/]util\.js$/,
          chunks: /^(Foo|Bar)$/,
          maxAsyncRequests: 2,
          minChunks: 2,
          name: 'Target',
          priority: 100,
        },
        lowerPriority: {
          test: /[\\/]util\.js$/,
          chunks: 'all',
          minChunks: 2,
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
        'NamedTargetOutsideFilterResidual',
        (compilation) => {
          compilation.hooks.afterSeal.tap(
            'NamedTargetOutsideFilterResidual',
            () => {
              const lowerPriorityChunk =
                compilation.namedChunks.get('lower_util');
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

              expect(lowerPriorityChunk).toBeDefined();
              expect(utilChunks).toHaveLength(1);
              expect(utilChunks).toContain(lowerPriorityChunk);
            },
          );
        },
      );
    },
  ],
};
