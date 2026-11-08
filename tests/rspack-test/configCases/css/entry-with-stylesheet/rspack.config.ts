import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  mode: 'development',
  entry: {
    main: ['./index.js', './style.css'],
  },
  output: {
    filename: '[name].js',
  },
  module: {
    rules: [{ test: /\.css$/, type: 'css/auto' }],
  },
});
