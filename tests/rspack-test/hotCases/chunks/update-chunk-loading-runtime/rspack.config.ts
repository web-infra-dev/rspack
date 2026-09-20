import { defineConfig } from '@rspack/cli';

export default defineConfig(({ config: _config }) => ({
  output: {
    filename: '[name].js',
  },
  optimization: {
    runtimeChunk: true,
    splitChunks: {
      chunks: 'all',
      minSize: 0,
    },
  },
}));
