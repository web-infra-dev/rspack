import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  output: {
    assetModuleFilename: 'images/file[ext]',
  },
  module: {
    generator: {
      asset: {
        emit: false,
      },
    },
    parser: {
      asset: {
        dataUrlCondition: {
          maxSize: 0,
        },
      },
    },
    rules: [
      {
        test: /\.png$/,
        type: 'asset',
      },
      {
        test: /\.jpg$/,
        type: 'asset',
      },
    ],
  },
});
