import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    splitChunks: {
      cacheGroups: {
        first: {
          test: /module1/,
          name: 'named',
          enforce: true,
          priority: 100,
        },
        second: {
          test: /module(1|2)/,
          name: 'named',
          enforce: true,
          priority: 50,
        },
      },
    },
  },
});
