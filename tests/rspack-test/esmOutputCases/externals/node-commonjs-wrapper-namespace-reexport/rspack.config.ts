import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    'webpack-sources': 'node-commonjs webpack-sources/lib/index.js',
  },
});
