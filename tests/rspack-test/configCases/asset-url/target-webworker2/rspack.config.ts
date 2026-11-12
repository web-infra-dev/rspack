import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  target: 'webworker',
  devtool: false,
  output: {
    assetModuleFilename: '[name][ext]',
    publicPath: '/',
  },
});
