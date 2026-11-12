import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  mode: 'production',
  devtool: false,
  experiments: {
    css: true,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/module',
      },
    ],
    parser: {
      'css/module': {
        namedExports: false,
        exportType: 'link',
      },
    },
  },
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
});
