/** @typedef {import("@rspack/core").Compiler} Compiler */

const leafChunkNames = ['Foo', 'Bar', 'Other', 'Other2'];

/**
 * @param {string} name
 * @param {RegExp} highPriorityChunks
 * @param {number} expectedUtilChunkCount
 * @param {string[]} expectedLeafChunksWithUtil
 * @param {number} minSize
 * @param {boolean} expectResidualModulesTogether
 * @returns {import("@rspack/core").Configuration}
 */
function createConfig(
  name,
  highPriorityChunks,
  expectedUtilChunkCount,
  expectedLeafChunksWithUtil,
  minSize = 0,
  expectResidualModulesTogether = false,
) {
  return {
    name,
    mode: 'development',
    target: 'web',
    output: {
      filename: `${name}-[name].js`,
      chunkFilename: `${name}-[name].js`,
    },
    optimization: {
      runtimeChunk: 'single',
      splitChunks: {
        minSize,
        cacheGroups: {
          shared_util: {
            test: /[\\/]util\.js$/,
            chunks: highPriorityChunks,
            minSize: 0,
            minChunks: 2,
            name: 'shared_util',
            priority: 100,
          },
          default: {
            chunks: 'all',
            minChunks: 2,
          },
        },
      },
    },
    plugins: [
      /** @this {Compiler} */
      function () {
        this.hooks.compilation.tap(name, (compilation) => {
          compilation.hooks.afterSeal.tap(name, () => {
            const utilChunks = [];
            const helperChunks = [];

            for (const chunk of compilation.chunks) {
              const moduleIdentifiers = Array.from(
                compilation.chunkGraph.getChunkModulesIterable(chunk),
              ).map((module) => module.identifier().replaceAll('\\', '/'));
              const containsUtil = moduleIdentifiers.some((identifier) =>
                identifier.endsWith('/util.js'),
              );
              const containsHelper = moduleIdentifiers.some((identifier) =>
                identifier.endsWith('/helper.js'),
              );

              if (containsUtil) {
                utilChunks.push(chunk);
              }
              if (containsHelper) {
                helperChunks.push(chunk);
              }
            }

            const sharedUtilChunk = compilation.namedChunks.get('shared_util');
            expect(sharedUtilChunk).toBeDefined();
            expect(utilChunks).toContain(sharedUtilChunk);
            expect(utilChunks).toHaveLength(expectedUtilChunkCount);
            expect(
              utilChunks
                .map((chunk) => chunk.name)
                .filter((chunkName) => leafChunkNames.includes(chunkName))
                .sort(),
            ).toEqual([...expectedLeafChunksWithUtil].sort());

            if (expectResidualModulesTogether) {
              const residualUtilChunk = utilChunks.find(
                (chunk) => chunk !== sharedUtilChunk,
              );
              expect(residualUtilChunk).toBeDefined();
              expect(helperChunks).toContain(residualUtilChunk);
            }
          });
        });
      },
    ],
  };
}

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  createConfig('partial-residual', /^(Foo|Bar)$/, 2, []),
  createConfig('residual-below-min-chunks', /^(Foo|Bar|Other)$/, 2, ['Other2']),
  createConfig('no-residual', /^(Foo|Bar|Other|Other2)$/, 1, []),
  createConfig('regroup-residual', /^(Foo|Bar)$/, 2, [], 30, true),
];
