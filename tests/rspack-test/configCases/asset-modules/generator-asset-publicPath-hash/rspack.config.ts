import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  output: {
    assetModuleFilename: '[contenthash:10].file[ext]',
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
        publicPath: '[contenthash]/assets/',
      },
    },
  },
});
