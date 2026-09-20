import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  devtool: 'source-map',
  externals: ['source-map'],
  entry: './index.js',
});
