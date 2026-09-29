import { defineConfig, definePlugin } from '@rspack/cli';
import { NormalModule } from '@rspack/core';

const pluginName = 'CheckModuleChunks';

export default defineConfig(
  [1, 4, 5, 9].flatMap((count) =>
    [false, true].map((split) =>
      defineConfig({
        mode: 'production',
        target: 'node',
        entry: Object.fromEntries(
          Array.from({ length: count }, (_, i) => [`entry${i}`, './shared.js']),
        ),
        output: { filename: '[name].js' },
        optimization: {
          minimize: false,
          inlineExports: false,
          concatenateModules: false,
          mergeDuplicateChunks: false,
          splitChunks: split
            ? {
                cacheGroups: {
                  default: false,
                  defaultVendors: false,
                  shared: {
                    test(module, { chunkGraph }) {
                      if (
                        !(module instanceof NormalModule) ||
                        !/[\\/]shared\.js$/.test(module.resource || '')
                      )
                        return false;
                      expect(chunkGraph.getModuleChunks(module)).toHaveLength(
                        count,
                      );
                      return true;
                    },
                    chunks: 'all',
                    name: 'shared',
                    enforce: true,
                  },
                },
              }
            : false,
        },
        plugins: [
          definePlugin({
            apply(compiler) {
              compiler.hooks.compilation.tap(pluginName, (compilation) => {
                const check = (expected: number) => {
                  const module = [...compilation.modules].find(
                    (module) =>
                      module instanceof NormalModule &&
                      /[\\/]shared\.js$/.test(module.resource || ''),
                  );
                  expect(module).toBeDefined();
                  const chunks = compilation.chunkGraph.getModuleChunks(
                    module!,
                  );
                  expect(chunks).toHaveLength(expected);
                  expect(new Set(chunks.map((chunk) => chunk.name)).size).toBe(
                    expected,
                  );
                  for (const chunk of chunks) {
                    expect(
                      compilation.chunkGraph
                        .getChunkModules(chunk)
                        .some(
                          (member) =>
                            member.identifier() === module!.identifier(),
                        ),
                    ).toBe(true);
                  }
                };
                compilation.hooks.optimizeTree.tap(pluginName, () =>
                  check(split ? 1 : count),
                );
              });
            },
          }),
        ],
      }),
    ),
  ),
);
