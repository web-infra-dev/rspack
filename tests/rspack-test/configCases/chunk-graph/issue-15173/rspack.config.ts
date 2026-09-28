import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    entryA: './entries/entryA.js',
    entryB: './entries/entryB.js',
  },
  output: {
    filename: '[name].js',
  },
});
