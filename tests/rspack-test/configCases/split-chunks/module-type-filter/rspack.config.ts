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
    splitChunks: {
      cacheGroups: {
        json: {
          name: 'json',
          type: 'json',
          chunks: 'all',
          enforce: true,
        },
      },
    },
  },
});
