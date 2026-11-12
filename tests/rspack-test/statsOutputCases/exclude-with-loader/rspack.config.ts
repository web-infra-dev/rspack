import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './index',
  output: {
    filename: 'bundle.js',
  },
  stats: {
    assets: true,
    modules: true,
    excludeModules: ['node_modules', 'exclude'],
    excludeAssets: [/\.json/],
  },
  module: {
    rules: [
      {
        test: /\.txt/,
        loader: 'raw-loader',
      },
      {
        test: /\.json/,
        loader: 'file-loader',
        options: {
          name: '[sha256:hash:8].[ext]',
        },
        type: 'javascript/auto',
      },
    ],
  },
});
