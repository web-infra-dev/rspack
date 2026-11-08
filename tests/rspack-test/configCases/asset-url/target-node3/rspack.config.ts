import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  target: 'node',
  devtool: false,
  output: {
    assetModuleFilename: '[name][ext]',
    publicPath: 'https://example.com/',
  },
});
