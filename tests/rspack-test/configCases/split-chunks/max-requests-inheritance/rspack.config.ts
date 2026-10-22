import { defineConfig, definePlugin } from '@rspack/cli';
import type {
  OptimizationSplitChunksCacheGroup,
  OptimizationSplitChunksOptions,
} from '@rspack/core';

function createConfig(
  name: string,
  options: OptimizationSplitChunksOptions,
  group: OptimizationSplitChunksCacheGroup,
  expectedChunks: string[],
) {
  return defineConfig({
    name,
    mode: 'production',
    target: 'node',
    output: {
      filename: `${name}-[name].js`,
      chunkFilename: `${name}-[name].js`,
    },
    optimization: {
      minimize: false,
      concatenateModules: false,
      splitChunks: {
        chunks: 'all',
        minSize: 0,
        enforceSizeThreshold: Infinity,
        ...options,
        cacheGroups: {
          default: false,
          defaultVendors: false,
          shared: {
            test: /[\\/]shared\.js$/,
            name: (module) =>
              module.identifier().endsWith('?initial')
                ? 'shared-initial'
                : 'shared-async',
            ...group,
          },
        },
      },
    },
    plugins: [
      definePlugin(function () {
        this.hooks.compilation.tap(name, (compilation) => {
          compilation.hooks.afterSeal.tap(name, () => {
            const chunks = Array.from(compilation.chunks)
              .map((chunk) => chunk.name)
              .filter((name) => name?.startsWith('shared-'))
              .sort();
            expect({ [name]: chunks }).toEqual({ [name]: expectedChunks });
          });
        });
      }),
    ],
  });
}

const limited = { maxInitialRequests: 1, maxAsyncRequests: 1 };

export default defineConfig([
  createConfig(
    'inherit-initial',
    { maxInitialRequests: 1, maxAsyncRequests: Infinity },
    {},
    ['shared-async'],
  ),
  createConfig(
    'inherit-async',
    { maxInitialRequests: Infinity, maxAsyncRequests: 1 },
    {},
    ['shared-initial'],
  ),
  createConfig('override-initial', limited, { maxInitialRequests: 2 }, [
    'shared-initial',
  ]),
  createConfig('override-async', limited, { maxAsyncRequests: 2 }, [
    'shared-async',
  ]),
  createConfig(
    'restrict-group',
    { maxInitialRequests: 2, maxAsyncRequests: 2 },
    limited,
    [],
  ),
  createConfig('enforce', limited, { enforce: true }, [
    'shared-async',
    'shared-initial',
  ]),
  createConfig(
    'enforce-explicit-limits',
    limited,
    { enforce: true, ...limited },
    [],
  ),
]);
