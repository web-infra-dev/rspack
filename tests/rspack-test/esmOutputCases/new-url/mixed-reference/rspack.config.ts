import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    parser: {
      javascript: {
        url: 'new-url-relative',
      },
    },
    rules: [
      {
        test: /\.txt$/,
        type: 'asset/resource',
      },
    ],
  },
  output: {
    assetModuleFilename: '[name][ext]',
  },
});
