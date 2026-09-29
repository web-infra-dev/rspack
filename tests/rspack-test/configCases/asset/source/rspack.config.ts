import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.txt$/,
        type: 'asset/source',
      },
    ],
  },
});
