import { defineConfig } from '@rspack/cli';

const stats = {
  hash: false,
  timings: false,
  builtAt: false,
  assets: false,
  chunks: true,
  chunkOrigins: true,
  modules: false,
};

const config = defineConfig({
  mode: 'production',
  entry: {
    main: './',
  },
  optimization: {
    splitChunks: {
      minSize: 100,
      cacheGroups: {
        vendors: {
          test: /[\\/]node_modules[\\/]/,
          chunks: 'async',
          enforce: true,
          name: 'vendors',
        },
      },
    },
  },
});

export default defineConfig([
  {
    stats: { entrypoints: false, chunkGroups: true, ...stats },
    output: {
      filename: 'a-[name].js',
    },
    ...config,
  },
  {
    stats: { entrypoints: true, chunkGroups: true, ...stats },
    output: {
      filename: 'b-[name].js',
    },
    ...config,
  },
]);
