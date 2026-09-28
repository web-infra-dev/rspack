import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  devtool: false,
  entry: {
    'css-entry': './entry.css',
    main: './index.js',
  },
  output: {
    filename: '[name].js',
    cssFilename: '[name].css',
  },
  node: {
    __dirname: false,
  },
});
