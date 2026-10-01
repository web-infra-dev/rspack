import { defineConfig } from '@rspack/cli';
import {
  type Chunk,
  type Configuration,
  type Module,
  rspack,
} from '@rspack/core';

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
                // Bound distinct sets: {A,B,C}, or {A,B} and {A,B,C}.
                expect(counters[group]).toBeLessThanOrEqual(
                  shape === 'same' ? 3 : 5,
                );
              }
            });
          }
        },
      },
    ],
  };
}

// Compare within one compilation: independent compilers have different
// ChunkUkeys and can legitimately have different hash-set iteration orders.
function createOrderConfig(shape: 'same' | 'overlap'): Configuration {
  const orders = [new Map<string, string[]>(), new Map<string, string[]>()];
  const namespace = `${shape}-order`;
  return {
    name: namespace,
    mode: 'development',
    target: 'node',
    entry: { [namespace]: `./${shape}/index.js` },
    devtool: false,
    output: { filename: '[name].js', chunkFilename: '[name].js' },
    optimization: {
      minimize: false,
      concatenateModules: false,
      usedExports: true,
      splitChunks: {
        usedExports: true,
        minSize: 0,
        cacheGroups: {
          default: false,
          defaultVendors: false,
          ...Object.fromEntries(
            [0, 1].map((group) => [
              `group${group}`,
              {
                test: /shared-\d+\.js$/,
                priority: 0,
                minSize: 0,
                minChunks: 2,
                enforce: true,
                chunks:
                  group === 0
                    ? /-route-[ab]$/
                    : (chunk: Chunk) => /-route-[ab]$/.test(chunk.name ?? ''),
                name(module: Module, chunks: Chunk[]) {
                  // Deliberately order-sensitive, matching the unchanged native
                  // path rather than sorting the callback's chunks array.
                  expect(chunks).toHaveLength(2);
                  const order = chunks.map((chunk) => chunk.name).join('~');
                  const key = module.identifier();
                  const previous = orders[group].get(key) ?? [];
                  previous.push(order);
                  orders[group].set(key, previous);
                  return `${namespace}-${group}-${order}`;
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
          compiler.hooks.done.tap('AssertSelectorOrder', () => {
            const normalize = (value: Map<string, string[]>) =>
              [...value].map(([key, calls]) => [key, [...calls].sort()]).sort();
            // Scheduling of module callbacks is not part of the contract; the
            // chunks array order for each callback must match the native path.
            expect(orders[0].size).toBe(8);
            expect(normalize(orders[1])).toEqual(normalize(orders[0]));
          });
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
  createOrderConfig('same'),
  createOrderConfig('overlap'),
]);
