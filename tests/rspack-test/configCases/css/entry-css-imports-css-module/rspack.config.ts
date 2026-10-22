import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  mode: 'development',
  entry: {
    main: './index.css',
    test: './test.js',
  },
  output: {
    filename: '[name].js',
  },
  module: {
    rules: [{ test: /\.css$/, type: 'css/auto' }],
  },
});
