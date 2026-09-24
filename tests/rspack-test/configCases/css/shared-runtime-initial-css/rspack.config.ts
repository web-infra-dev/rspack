import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './main.js',
    other: './feature.js',
  },
  output: {
    filename: '[name].js',
    chunkFilename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
    runtimeChunk: 'single',
    splitChunks: {
      cacheGroups: {
        sharedCss: {
          test: /feature\.css$/,
          name: 'shared-css',
          chunks: 'all',
          enforce: true,
        },
      },
    },
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css',
        parser: {
          exportType: 'link',
        },
      },
    ],
  },
});
