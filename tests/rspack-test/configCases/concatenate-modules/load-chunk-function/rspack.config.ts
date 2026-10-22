import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    entry1: './entry1',
    entry2: './entry2',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    concatenateModules: true,
  },
});
