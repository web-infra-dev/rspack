import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index',
  module: {
    rules: [
      {
        test: /\.js$/,
        use: ['./test-loader.mjs'],
      },
    ],
  },
  stats: {
    all: false,
    loggingDebug: [/TestLoader/],
  },
});
