import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.(png|svg)$/,
        type: 'asset/resource',
      },
      {
        test: /\.jpg$/,
        type: 'asset/resource',
      },
    ],
  },
});
