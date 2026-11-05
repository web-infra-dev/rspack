import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    a: './a.css',
    b: './b.css',
    c: './c.js',
    main: './index.js',
  },
  output: {
    filename: '[name].js',
    cssFilename: '[name].css',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css',
      },
    ],
  },
  optimization: {
    runtimeChunk: 'single',
  },
});
