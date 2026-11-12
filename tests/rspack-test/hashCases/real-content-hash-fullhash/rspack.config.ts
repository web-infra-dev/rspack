import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './index',
  optimization: {
    realContentHash: true,
  },
  output: {
    filename: '[fullhash].js',
  },
});
