import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  output: {
    assetModuleFilename: 'images/file[ext]',
  },
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset',
      },
      {
        test: /\.html$/,
        type: 'asset/resource',
        generator: {
          filename: 'static/index.html',
        },
      },
    ],
  },
});
