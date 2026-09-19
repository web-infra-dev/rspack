import { defineConfig } from '@rspack/cli';

export default defineConfig({
  devtool: 'source-map',
  optimization: {
    minimize: true,
    concatenateModules: false,
  },
  externals: ['source-map'],
  externalsType: 'commonjs',
});
