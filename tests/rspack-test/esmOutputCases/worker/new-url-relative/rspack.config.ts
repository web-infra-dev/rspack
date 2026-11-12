import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  module: {
    parser: {
      javascript: {
        worker: {
          url: 'new-url-relative',
        },
      },
    },
  },
  optimization: {
    runtimeChunk: false,
  },
});
