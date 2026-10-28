import { defineConfig } from '@rspack/cli';

const base = defineConfig({
  mode: 'production',
  optimization: {
    moduleIds: 'named',
    chunkIds: 'named',
    runtimeChunk: true,
    splitChunks: {
      minSize: 0,
      chunks: 'all',
      cacheGroups: {
        all: {
          priority: -30,
        },
      },
    },
  },
  stats: {
    entrypoints: true,
    assets: true,
    modules: true,
  },
});

export default defineConfig([
  {
    entry: './a.js',
    output: {
      filename: 'a-[name]-[chunkhash].js',
    },
    ...base,
  },
  {
    entry: './b.js',
    output: {
      filename: 'b-[name]-[chunkhash].js',
    },
    ...base,
  },
  {
    entry: './c.js',
    output: {
      filename: 'c-[name]-[chunkhash].js',
    },
    ...base,
  },
]);
