import { defineConfig, definePlugin } from '@rspack/cli';

/**
 * @param name
 * @param minSizeReduction
 * @param expectSingleCopy
 * @returns {import("@rspack/core").Configuration}
 */
function createConfig(
  name: string,
  minSizeReduction: number,
  expectSingleCopy: boolean,
) {
  return defineConfig({
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
          // and this candidate can only extract Foo's remaining copy. With a positive requirement,
          // highPriority reduces no source chunk, so this candidate can still merge both copies.
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
      definePlugin(function () {
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
            expect(samePriorityChunk).toBeDefined();
            expect(lowerPriorityChunk).toBeNull();
            expect(utilChunks).toContain(samePriorityChunk);
            expect(
              Array.from(
                samePriorityChunk!.groupsIterable,
                (group) => group.name,
              ).sort(),
            ).toEqual(expectSingleCopy ? ['Foo', 'ReusableUtil'] : ['Foo']);
            if (expectSingleCopy) {
              expect(utilChunks).toHaveLength(1);
            } else {
              expect(utilChunks).toHaveLength(2);
              expect(utilChunks).toContain(reusableChunk);
            }
          });
        });
      }),
    ],
  });
}

export default defineConfig([
  createConfig('preserve-reused-destination', 0, false),
  createConfig('revalidate-min-size-reduction', 1, true),
]);
