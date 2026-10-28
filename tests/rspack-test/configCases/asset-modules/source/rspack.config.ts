import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.svg$/,
        type: 'asset/source',
      },
    ],
  },
});
