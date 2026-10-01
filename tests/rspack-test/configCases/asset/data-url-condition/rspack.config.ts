import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset',
      },
    ],
    parser: {
      asset: {
        dataUrlCondition: {
          maxSize: 100 * 1024,
        },
      },
    },
  },
});
