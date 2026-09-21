import { defineConfig } from '@rspack/cli';

const stats = {
  hash: false,
  timings: false,
  builtAt: false,
  assets: false,
  chunks: true,
  chunkRelations: true,
  chunkOrigins: true,
  entrypoints: true,
  modules: false,
};

export default defineConfig({
  name: 'default',
  mode: 'production',
  entry: {
    main: './',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    splitChunks: {
      minSize: 0, // enforce all
    },
  },
  stats,
});
