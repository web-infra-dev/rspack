import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  entry: './src/index',
  target: 'web',
  node: false,
  output: {
    publicPath: '/',
  },
  module: {
    rules: [
      {
        test: /\.less$/,
        use: [
          {
            loader: 'less-loader',
          },
        ],
        type: 'css',
        generator: {
          exportsOnly: false,
        },
      },
    ],
  },
});
