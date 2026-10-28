import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  output: {
    filename: 'main.js',
    chunkFilename: 'fallback-[id].js',
  },
  optimization: {
    splitChunks: {
      cacheGroups: {
        encoded: {
          chunks: 'async',
          test: /async\.js$/,
          name: 'encoded',
          filename: 'split-[name]-[fullhash:base64:4].js',
          enforce: true,
        },
      },
    },
  },
});
