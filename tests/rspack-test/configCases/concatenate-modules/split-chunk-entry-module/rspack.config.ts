import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './index',
  },
  target: 'web',
  output: {
    filename: '[name].js',
  },
  optimization: {
    concatenateModules: false,
    chunkIds: 'named',
    runtimeChunk: 'single',
    splitChunks: {
      minSize: 0,
      cacheGroups: {
        common: {
          chunks: 'initial',
          minChunks: 1,
        },
      },
    },
  },
});
