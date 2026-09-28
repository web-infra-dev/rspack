import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.(png|svg|jpg)$/,
        type: 'asset',
      },
    ],
  },
});
