import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  node: {
    __dirname: false,
    __filename: false,
  },
  devtool: 'source-map',
});
