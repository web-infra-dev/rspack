import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  devtool: 'eval',
  entry: './index.js',
  output: {
    filename: '[contenthash].js',
  },
  stats: {
    all: false,
    assets: true,
  },
});
