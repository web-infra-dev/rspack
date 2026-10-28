import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    path: 'node-commonjs path',
  },
  output: {
    module: true,
  },
  devtool: 'eval-source-map',
  target: 'node',
});
