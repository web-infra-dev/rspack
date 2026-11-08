import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  output: {
    assetModuleFilename: 'file[ext]',
    publicPath: 'assets/',
  },
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset',
        generator: {
          publicPath: '',
        },
      },
    ],
  },
});
