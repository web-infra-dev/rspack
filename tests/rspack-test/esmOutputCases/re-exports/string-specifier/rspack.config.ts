import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.json$/,
        type: 'json',
      },
    ],
  },
  optimization: {
    splitChunks: {
      cacheGroups: {
        lib: {
          test: /lib\.js/,
        },
      },
    },
  },
});
