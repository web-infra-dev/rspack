import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  output: {
    assetModuleFilename: 'images/file[ext]',
  },
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/resource',
        generator: {
          emit: false,
        },
      },
      {
        test: /\.jpg$/,
        type: 'asset/resource',
      },
    ],
  },
});
