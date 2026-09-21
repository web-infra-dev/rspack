import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  mode: 'development',
  externalsPresets: { web: false },
  output: {
    publicPath: '',
    assetModuleFilename: '[name][ext]',
  },
  module: {
    rules: [
      { test: /\.css$/, type: 'css' },
      { test: /\.svg$/, type: 'asset/resource' },
    ],
  },
});
