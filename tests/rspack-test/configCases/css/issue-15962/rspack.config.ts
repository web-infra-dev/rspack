import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  mode: 'development',
  devtool: false,
  node: {
    __dirname: false,
    __filename: false,
  },
  module: {
    generator: {
      'css/module': {
        localIdentName: 'LOCAL-[local]',
      },
      'css/global': {
        localIdentName: 'LOCAL-[local]',
      },
    },
    rules: [
      {
        test: /\.global\.css$/,
        type: 'css/global',
      },
      {
        test: /\.modules\.css$/,
        type: 'css/module',
      },
    ],
  },
  experiments: {
    css: true,
  },
});
