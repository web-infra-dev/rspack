import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  devtool: false,
  module: {
    parser: {
      asset: {
        dataUrlCondition() {
          return true;
        },
      },
    },
    rules: [
      {
        test: /file-global\.txt$/,
        type: 'asset',
      },
      {
        test: /file-local\.txt$/,
        type: 'asset',
        parser: {
          dataUrlCondition() {
            return false;
          },
        },
      },
    ],
  },
});
