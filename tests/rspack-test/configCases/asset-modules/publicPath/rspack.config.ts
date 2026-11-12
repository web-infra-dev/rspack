import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  output: {
    publicPath: 'assets/',
    assetModuleFilename: 'file[ext]',
    environment: {
      templateLiteral: true,
    },
  },
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset',
      },
    ],
  },
});
