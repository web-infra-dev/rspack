import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    assetModuleFilename: 'assets/[name][ext]',
  },
  module: {
    rules: [
      {
        test: /\.png$/,
        use: './loader.mjs',
        type: 'asset/resource',
      },
    ],
  },
});
