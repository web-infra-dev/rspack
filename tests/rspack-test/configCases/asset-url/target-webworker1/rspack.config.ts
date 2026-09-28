import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  target: 'webworker',
  devtool: false,
  output: {
    filename: 'deep/path/[name].js',
    assetModuleFilename: '[path][name][ext]',
    publicPath: '',
  },
});
