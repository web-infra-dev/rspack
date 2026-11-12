import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  stats: {
    entrypoints: true,
    assets: true,
    modules: true,
  },
});
