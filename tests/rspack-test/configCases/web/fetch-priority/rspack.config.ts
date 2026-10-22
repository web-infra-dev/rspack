import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  output: {
    chunkFilename: '[name].js',
    crossOriginLoading: 'anonymous',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  optimization: {
    minimize: false,
    splitChunks: {
      minSize: 1,
    },
  },
});
