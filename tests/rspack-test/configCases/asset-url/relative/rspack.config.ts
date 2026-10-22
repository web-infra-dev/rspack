import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  devtool: false,
  output: {
    assetModuleFilename: '[name][ext][query][fragment]',
    publicPath: 'public/',
  },
  module: {
    parser: {
      javascript: {
        url: 'relative',
      },
    },
  },
});
