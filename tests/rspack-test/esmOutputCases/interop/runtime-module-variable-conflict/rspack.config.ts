import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /foo\.mjs$/,
        type: 'asset/resource',
        generator: {
          importMode: 'preserve',
        },
      },
    ],
  },
  optimization: {
    runtimeChunk: false,
  },
});
