import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.png$/,
        generator: {
          filename: '[name][ext]',
        },
        type: 'asset/resource',
      },
    ],
  },
});
