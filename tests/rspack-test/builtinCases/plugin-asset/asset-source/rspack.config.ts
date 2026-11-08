import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.txt$/,
        type: 'asset/source',
      },
    ],
  },
});
