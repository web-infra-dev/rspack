import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      { test: /\.less$/, type: 'asset/source', use: 'builtin:less-loader' },
    ],
  },
});
