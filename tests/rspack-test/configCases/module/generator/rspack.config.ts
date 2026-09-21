import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  output: {
    publicPath: '/',
    assetModuleFilename: 'asset/[name][ext]',
  },
  module: {
    rules: [
      {
        test: /\.png$/,
        resourceQuery: /custom/,
        type: 'asset/resource',
        generator: {
          filename: 'custom-asset/[name][ext]',
        },
      },
      {
        test: /\.svg$/,
        resourceQuery: /non-custom/,
        type: 'asset/resource',
      },
    ],
  },
});
