import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: {
    __dirname: false,
    __filename: false,
  },
  output: {
    publicPath: '/',
    cssFilename: 'css/[name].css',
  },
  resolve: {
    alias: {
      '@': import.meta.dirname,
    },
  },
  module: {
    generator: {
      'css/auto': {
        exportsOnly: false,
      },
    },
    rules: [
      {
        test: /\.png$/i,
        type: 'asset',
        generator: {
          filename: 'image/[name].[contenthash:8][ext]',
        },
      },
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
