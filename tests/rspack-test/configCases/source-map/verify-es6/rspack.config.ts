import { defineConfig } from '@rspack/cli';

export default defineConfig({
  devtool: 'source-map',
  externals: ['source-map'],
  externalsType: 'commonjs',
});
