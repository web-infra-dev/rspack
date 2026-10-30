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
  module: {
    generator: {
      'css/auto': {
        localIdentName: '[local]-scoped',
      },
    },
    rules: [
      {
        test: /\.module\.css$/,
        type: 'css/auto',
      },
    ],
  },
  optimization: {
    minimize: false,
  },
});
