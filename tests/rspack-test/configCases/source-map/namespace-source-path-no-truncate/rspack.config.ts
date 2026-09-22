import { defineConfig } from '@rspack/cli';

export default defineConfig({
  node: {
    __dirname: false,
    __filename: false,
  },
  devtool: 'source-map',
  optimization: {
    minimize: true,
  },
});
