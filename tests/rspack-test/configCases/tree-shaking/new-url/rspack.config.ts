import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.svg$/,
        type: 'asset/resource',
      },
    ],
  },

  optimization: {
    sideEffects: true,
  },
  output: {
    chunkFilename: '[name].js',
  },
  externalsPresets: {
    node: true,
  },
});
