import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  target: 'web',
  devtool: false,
  output: {
    assetModuleFilename: '[name][ext]',
    publicPath: '/path2/',
  },
});
