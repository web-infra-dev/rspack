import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  mode: 'development',
  devtool: 'eval',
  stats: {
    assets: true,
    modules: true,
  },
  output: {
    filename: 'bundle.js',
  },
});
