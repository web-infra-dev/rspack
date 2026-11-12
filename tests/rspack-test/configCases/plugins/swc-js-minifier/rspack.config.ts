import { defineConfig } from '@rspack/cli';
import { SwcJsMinimizerRspackPlugin } from '@rspack/core';

export default defineConfig({
  entry: {
    main: ['./index.js'],
    extract: ['./extract.js'],
    'no-extract': ['./no-extract.js'],
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    new SwcJsMinimizerRspackPlugin({
      extractComments: true,
      minimizerOptions: {
        format: {
          comments: false,
        },
      },
      include: ['extract.js', 'no-extract.js'],
    }),
  ],
});
