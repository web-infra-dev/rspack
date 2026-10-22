import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'none',
  target: 'node',
  output: {
    assetModuleFilename: '[name][ext]',
  },
  module: {
    rules: [
      {
        test: /\.jpg$/,
        type: 'asset/resource',
      },
    ],
  },
});
