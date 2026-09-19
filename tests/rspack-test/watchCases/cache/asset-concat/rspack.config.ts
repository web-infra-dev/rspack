import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  cache: true,
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/inline',
      },
      {
        test: /\.jpg$/,
        type: 'asset/resource',
      },
      {
        test: /\.svg$/,
        type: 'asset',
      },
    ],
  },
  optimization: {
    concatenateModules: true,
  },
});
