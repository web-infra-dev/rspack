import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    'entry?query': './index.js',
  },
  output: {
    filename: '[contenthash].js',
    chunkFilename: () => '[name].js',
  },
});
