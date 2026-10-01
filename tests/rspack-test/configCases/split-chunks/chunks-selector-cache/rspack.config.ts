import { defineConfig } from '@rspack/cli';
import { type Chunk, type Configuration, rspack } from '@rspack/core';

const outputs = new Map<string, Map<string, Map<string, Buffer>>>();

function createConfig(
  shape: 'same' | 'overlap',
  native: boolean,
): Configuration {
  const counters = [0, 0];
  const seen = [new Set<Chunk>(), new Set<Chunk>()];
  return {
    name: `${shape}-${native ? 'native' : 'function'}`,
    mode: 'development',
    target: 'node',
    entry: { [shape]: `./${shape}/index.js` },
    devtool: false,
    output: { filename: '[name].js', chunkFilename: '[name].js' },
    optimization: {
      minimize: false,
      concatenateModules: false,
      usedExports: false,
      splitChunks: {
        chunks: 'all',
        minSize: 0,
        cacheGroups: {
          default: false,
          defaultVendors: false,
          ...Object.fromEntries(
            [0, 1].map((group) => [
              `group${group}`,
              {
                test: /shared-\d+\.js$/,
                name: `${shape}-shared-${group}`,
                priority: 0,
                minSize: 0,
                minChunks: 2,
                enforce: true,
                chunks: native
                  ? group === 0
                    ? 'all'
                    : 'async'
                  : (chunk: Chunk) => {
                      counters[group]++;
                      seen[group].add(chunk);
                      return group === 0 || !chunk.canBeInitial();
                    },
              },
            ]),
          ),
        },
      },
    },
    plugins: [
      {
        apply(compiler) {
          compiler.hooks.thisCompilation.tap(
            'CompareSelectorOutput',
            (compilation) => {
              compilation.hooks.processAssets.tap(
                {
                  name: 'CompareSelectorOutput',
                  stage: rspack.Compilation.PROCESS_ASSETS_STAGE_SUMMARIZE,
                },
                () => {
                  const output = new Map(
                    Object.entries(compilation.assets).map(([name, source]) => [
                      name,
                      Buffer.from(source.source()),
                    ]),
                  );
                  let pair = outputs.get(shape);
                  if (!pair) outputs.set(shape, (pair = new Map()));
                  pair.set(native ? 'native' : 'function', output);
                  if (pair.size === 2) {
                    const reference = pair.get('native')!;
                    const selected = pair.get('function')!;
                    expect([...selected.keys()].sort()).toEqual(
                      [...reference.keys()].sort(),
                    );
                    for (const [name, source] of selected) {
                      expect(source).toEqual(reference.get(name));
                    }
                  }
                },
              );
            },
          );
          if (!native) {
            compiler.hooks.done.tap('AssertSelectorMemo', () => {
              for (let group = 0; group < 2; group++) {
                expect(seen[group].size).toBe(3);
                // {A,B,C}, or both {A,B} and {A,B,C}; not a per-chunk cache.
                expect(counters[group]).toBeLessThanOrEqual(
                  shape === 'same' ? 3 : 5,
                );
                if (shape === 'overlap')
                  expect(counters[group]).toBeGreaterThan(3);
              }
            });
          }
        },
      },
    ],
  };
}

export default defineConfig([
  createConfig('same', true),
  createConfig('same', false),
  createConfig('overlap', true),
  createConfig('overlap', false),
]);
