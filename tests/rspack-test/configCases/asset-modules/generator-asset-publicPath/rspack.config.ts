import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  output: {
    assetModuleFilename: 'file[ext]',
  },
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset',
      },
    ],
    generator: {
      asset: {
        publicPath: 'assets/',
      },
    },
  },
});
