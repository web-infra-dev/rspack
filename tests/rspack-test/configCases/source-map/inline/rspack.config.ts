import { defineConfig } from '@rspack/cli';

export default defineConfig({
  devtool: 'inline-source-map',
  externals: ['source-map'],
  externalsType: 'commonjs',
});
