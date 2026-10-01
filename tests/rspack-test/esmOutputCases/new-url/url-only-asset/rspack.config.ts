import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    parser: {
      javascript: {
        url: 'new-url-relative',
      },
    },
  },
  output: {
    assetModuleFilename: '[name][ext]',
  },
});
