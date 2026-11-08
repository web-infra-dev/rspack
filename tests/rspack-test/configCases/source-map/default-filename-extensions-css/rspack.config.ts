import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  output: {
    filename: 'bundle0.css',
  },
  node: {
    __dirname: false,
    __filename: false,
  },
  devtool: 'source-map',
});
