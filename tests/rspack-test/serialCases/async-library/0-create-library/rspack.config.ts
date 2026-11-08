import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './a.js',
  output: {
    module: true,
    filename: 'lib.js',
    library: {
      type: 'module',
    },
  },
  target: 'node14',
  optimization: {
    minimize: true,
  },
});
