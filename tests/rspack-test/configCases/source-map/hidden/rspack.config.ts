import { defineConfig } from '@rspack/cli';

export default defineConfig({
  devtool: 'hidden-source-map',
  externals: ['source-map'],
  externalsType: 'commonjs',
});
