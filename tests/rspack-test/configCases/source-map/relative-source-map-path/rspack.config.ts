import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  output: {
    chunkFilename: 'js/chunks/c.js',
  },
  devtool: 'source-map',
  externals: ['source-map'],
  externalsType: 'commonjs',
});
