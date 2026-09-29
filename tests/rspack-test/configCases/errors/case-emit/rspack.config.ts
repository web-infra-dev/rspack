import { defineConfig } from '@rspack/cli';
import { CaseSensitivePlugin } from '@rspack/core';

export default defineConfig({
  entry: {
    a: './index.js?1',
    A: './index.js?2',
  },
  output: {
    filename: '[name].js',
  },
  plugins: [new CaseSensitivePlugin()],
});
