import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    assetModuleFilename: 'images/[hash][ext]',
  },
  module: {
    rules: [
      {
        test: /\.(png|jpg|svg)$/,
        type: 'asset',
      },
    ],
  },
});
