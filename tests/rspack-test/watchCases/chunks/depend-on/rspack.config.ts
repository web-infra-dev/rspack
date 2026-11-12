import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    shared: './shared.js',
    index1: {
      import: './index1.js',
      dependOn: 'shared',
    },
    index2: {
      import: './index2.js',
      dependOn: 'shared',
    },
  },
  output: {
    filename: '[name].js',
  },
});
